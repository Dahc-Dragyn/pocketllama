use anyhow::Context;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_llama::ModelWeights;
use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokenizers::Tokenizer;

#[derive(Clone)]
pub struct LocalEngine {
    model: Arc<Mutex<ModelWeights>>,
    tokenizer: Arc<Tokenizer>,
}

impl LocalEngine {
    /// Loads GGUF weights onto the CPU and parses the tokenizer from the given paths
    pub fn load(model_path: &Path, tokenizer_path: &Path) -> anyhow::Result<Self> {
        let mut file = File::open(model_path)
            .with_context(|| format!("Failed to open GGUF model file: {:?}", model_path))?;

        // Ingest the GGUF file contents
        let content = candle_core::quantized::gguf_file::Content::read(&mut file)
            .context("Failed to parse GGUF file metadata")?;

        // Instantiate ModelWeights on CPU
        let model_weights = ModelWeights::from_gguf(content, &mut file, &Device::Cpu)
            .context("Failed to load quantized LLaMA weights from GGUF content")?;

        // Load the tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Tokenizer error: {}", e))
            .with_context(|| format!("Failed to load tokenizer JSON file: {:?}", tokenizer_path))?;

        Ok(Self {
            model: Arc::new(Mutex::new(model_weights)),
            tokenizer: Arc::new(tokenizer),
        })
    }

    /// Asynchronously streams generated tokens across a tokio mpsc channel
    pub async fn generate_stream(
        &self,
        prompt: &str,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> anyhow::Result<()> {
        let model_arc = Arc::clone(&self.model);
        let tokenizer_arc = Arc::clone(&self.tokenizer);
        let prompt_str = prompt.to_string();

        // Offload the heavy blocking tensor computations to spawn_blocking
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let mut model_lock = model_arc
                .lock()
                .map_err(|e| anyhow::anyhow!("Model mutex poisoned: {}", e))?;

            // Setup LogitsProcessor with a default seed and moderate temperature
            let mut logits_processor = LogitsProcessor::new(299792458, Some(0.7), Some(0.9));

            // Tokenize input prompt
            let tokens = tokenizer_arc
                .encode(prompt_str.as_str(), true)
                .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;
            let input_ids = tokens.get_ids();

            if input_ids.is_empty() {
                anyhow::bail!("Input prompt generated zero tokens");
            }

            // Detect EOS token IDs in tokenizer vocabulary
            let eos_token_id = tokenizer_arc
                .token_to_id("</s>")
                .or_else(|| tokenizer_arc.token_to_id("<|endoftext|>"))
                .or_else(|| tokenizer_arc.token_to_id("<|im_end|>"))
                .or_else(|| tokenizer_arc.token_to_id("<|end|>"))
                .unwrap_or(2); // Fallback to LLaMA standard EOS token ID (2)

            let mut tokens_list = input_ids.to_vec();
            let mut pos = 0;
            let max_tokens = 512;

            for i in 0..max_tokens {
                let input = if i == 0 {
                    // Prefill phase: feed the entire prompt
                    Tensor::new(tokens_list.as_slice(), &Device::Cpu)
                        .context("Failed to construct prefill tensor")?
                        .unsqueeze(0)
                        .context("Failed to unsqueeze prefill tensor")?
                } else {
                    // Autoregressive phase: feed only the single last generated token
                    let last_token = *tokens_list
                        .last()
                        .ok_or_else(|| anyhow::anyhow!("Token sequence is empty"))?;
                    Tensor::new(&[last_token], &Device::Cpu)
                        .context("Failed to construct autoregressive tensor")?
                        .unsqueeze(0)
                        .context("Failed to unsqueeze autoregressive tensor")?
                };

                // Compute logits using forward pass on CPU GGUF model
                let logits = model_lock
                    .forward(&input, pos)
                    .context("Model forward pass failed")?;

                // Advance prompt sequence context index
                pos += input.dim(1).context("Failed to read sequence dimension")?;

                // Extract logits for the last token in sequence
                let logits = logits.squeeze(0).context("Failed to squeeze logits")?;
                let seq_len = logits.dim(0).context("Failed to read seq dimension")?;
                let last_logits = logits
                    .get(seq_len - 1)
                    .context("Failed to read last logit row")?;

                // Sample next token ID
                let next_token = logits_processor
                    .sample(&last_logits)
                    .context("Logits sampling failed")?;

                // Check for end of sequence signal
                if next_token == eos_token_id {
                    break;
                }

                // Append token ID to context list
                tokens_list.push(next_token);

                // Decode token ID to String slice
                if let Ok(piece) = tokenizer_arc.decode(&[next_token], true) {
                    // Check if channel is still open before sending
                    if tx.blocking_send(piece).is_err() {
                        // Receiver dropped, stop generation gracefully
                        break;
                    }
                }
            }

            Ok(())
        })
        .await
        .context("Tokio spawn_blocking task join failed")?
    }
}
