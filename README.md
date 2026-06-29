# Pocket LLaMA - Desktop Edge AI Appliance (v1.0)

**Pocket LLaMA** is a high-performance, air-gapped, zero-dependency Edge AI appliance designed for execution of sub-4B parameter quantized GGUF models on standard CPU hardware. Built entirely in Rust using Hugging Face's `candle` ecosystem, Pocket LLaMA delivers a premium, single-binary execution environment that bypasses heavy virtualization and Python runtime overhead.

Pocket LLaMA can be run directly from an external thumb drive, serving both a beautiful embedded dark-theme Web interface and an industry-standard Google Gemini-compatible REST API.

---

## 🎯 Executive Summary

In mission-critical, air-gapped environments, access to cloud-based large language models is restricted due to strict security guidelines. Pocket LLaMA addresses this challenge by packaging model inference, multi-turn dialogue memory, API emulation, and a modern Web console into a **single, self-contained executable** under 10MB. 

* **Zero-Dependency Engine**: Compiles to a single static binary. No Python, CUDA toolchains, Docker, or external runtimes required.
* **Air-Gapped Compliance**: The embedded assets and inference engine run 100% offline. Zero external CDN queries, telemetry collection, or remote calls.
* **Quantized CPU Inference**: Highly optimized CPU matrix math utilizing quantized Hugging Face `candle` backends, specifically tuned for local models like LLaMA and Phi.

---

## 🚀 Core Capabilities

### 1. "Double-Click" Desktop Application UX
By evaluating CLI arguments, Pocket LLaMA intercepts zero-parameter invocations (e.g., when a user double-clicks `pocket-cli.exe` directly on a Windows thumb drive).
* Instantly spins up a background API and web server on standard port `8080`.
* Launches the user's system-default web browser pointing directly to the local HUD portal.
* Renders a premium, glassmorphic dark-theme console served straight from the binary's RAM without any Disk I/O.

### 2. Hugging Face Candle Inference Engine
* Integrates Hugging Face's `candle-core`, `candle-transformers`, and `candle-nn` environments.
* Bridges synchronous matrix calculation pipelines to asynchronous Tokio runtimes via thread-safe `mpsc` token channels, ensuring real-time UI typewriter transitions.

### 3. Strict Zero-Panic Error Policy
* Features an explicit `PocketError` enum mapped through `thiserror`, completely replacing raw `unwrap()` or `expect()` sequences.
* Guarantees high availability: network timeouts, model load failures, or invalid JSON payloads are handled safely without terminating the process.

### 4. Gemini-Compatible REST API Schema
To enable immediate drop-in replacement for existing AI tools, Pocket LLaMA hosts a lightweight Axum REST service emulating the Google Gemini JSON schema:
* Endpoint: `POST /v1beta/models/:model` (e.g. `POST /v1beta/models/pocket:generateContent`)
* Supports structured message histories (`{"contents": [{"role": "user", "parts": [{"text": "..."}]}]}`) and maps context turns to maintain state across HTTP client cycles.

---

## 🧩 Supported Model Architectures

While Pocket LLaMA accepts general `.gguf` file formats, the internal tensor parsing engine is currently optimized specifically for the **LLaMA neural network architecture family** (wired via `candle_transformers::models::quantized_llama::ModelWeights`). 

Please check your model's base architecture before loading it into the local engine:

### 🟢 Supported Models (Green Light)
These architectures map cleanly to the internal LLaMA tensor structures and compile/execute out of the box:
* **TinyLlama** (e.g., `TinyLlama-1.1B-Chat-v1.0`)
* **Meta LLaMA 2 & LLaMA 3** (e.g., LLaMA-2-7B-Chat, LLaMA-3-8B-Instruct)
* **Mistral v0.2** (e.g., Mistral-7B-Instruct-v0.2)
* **LLaMA Fine-Tunes** (e.g., OpenHermes-2.5-Mistral-7B, Vicuna-7B)

### 🔴 Unsupported Models (Red Light)
Attempting to load these model containers will trigger a safe `PocketError::EngineError` due to incompatible neural layering:
* **Microsoft Phi-3** (Phi architectures construct attention layers differently)
* **Google Gemma & Gemma 2** (Gemma containers utilize custom scaling and normalization layouts)
* **Qwen 2 / Qwen 2.5** (Qwen formats require distinct multi-head attention routing)

### 🛠️ Developer Upgrade Path
For developers looking to extend architecture compatibility, a **Universal Architecture Router** can be integrated inside the `pocket-engine` crate. By reading GGUF metadata headers programmatically on launch, the engine can dynamically route tensors to respective Hugging Face `candle` parsers (e.g., `candle_transformers::models::quantized_gemma::ModelWeights` or `quantized_phi3::ModelWeights`).

---

## 🛡️ Pre-Flight Hardware Guardrail

Executing local LLMs on standard CPU hardware can be highly intensive. To protect user systems from memory exhaustion and OS freezing, Pocket LLaMA implements a strict **Pre-Flight Hardware interlock**:

> [!IMPORTANT]
> **The 80% RAM Safety Lockout**
> * On launch, the engine uses the `sysinfo` crate to verify the system's available physical RAM.
> * If loading the requested GGUF model would exceed **80% of total physical RAM capacity**, the engine will halt immediately and prevent model loading.
> * This guardrail protects your operating system from aggressive disk swap cycles and freezing.

### Bypassing the Safety Guardrail
For power users operating on dedicated machinery or lightweight configurations:
* Append the `--force-ram` CLI flag on launch to bypass the interlock and proceed with loading.

---

## 📦 Deployment Guide

Deploying Pocket LLaMA to a target workstation is simple. Since the entire application is packaged into a to-go single binary, you can deploy it from a standard USB thumb drive:

1. **Copy the Binary**: Transfer the compiled `pocket-cli.exe` executable onto your target deployment thumb drive (e.g., drive `H:\`).
2. **Transfer Models**: Copy your quantized GGUF model files (e.g., `tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf`) and their respective JSON tokenizers to the drive.
3. **Execute**:
   * **Plug & Play Web UI**: Insert the thumb drive into the target workstation and double-click `pocket-cli.exe`. Your default browser will open automatically, presenting the fully functional Chat console.
   * **Developer Command Line**: Run commands from PowerShell or Command Prompt directly from the drive path.

> [!TIP]
> **Recommended Testing Model**
> We highly recommend downloading **`TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf`** for your first test run.
> With a sub-1GB footprint, it loads in seconds, operates with minimal RAM usage, and provides an ideal, highly responsive baseline to verify the real-time UI streaming and Gemini API capabilities on standard laptops.

---

## 💻 Command Reference

### 1. The Interactive CLI REPL
Run local CPU model inference directly inside your terminal with high-performance real-time streaming:
```powershell
.\pocket-cli.exe --model "H:\models\tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf" --tokenizer "H:\models\tokenizer.json"
```

* **CLI Commands**:
  * Type your prompt and press `Enter` to stream tokens to stdout.
  * Use `exit` or `quit` to close the REPL session gracefully.
  * Press `Ctrl+C` or `Ctrl+D` to abort immediately.

### 2. The Web HUD (Double-Click Simulator)
Launches the Axum server on port `8080` and opens the web-based chat console in your default browser:
```powershell
# Zero arguments initiates default Desktop Appliance mode
.\pocket-cli.exe
```
*Alternatively, force Web UI mode with parameters:*
```powershell
.\pocket-cli.exe --ui --model "H:\models\tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf" --tokenizer "H:\models\tokenizer.json"
```

### 3. API Server Mode
Host the Gemini-compatible REST server without launching the browser (perfect for headless/background services):
```powershell
.\pocket-cli.exe --serve --model "H:\models\tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf" --tokenizer "H:\models\tokenizer.json"
```
*To test in offline mock mode (perfect for E2E integration validations without models loaded):*
```powershell
.\pocket-cli.exe --serve
```

---

## 🛠️ Build and Compilation Instructions

To build a highly optimized production binary containing all embedded HTML/CSS/JS assets inside a single `.exe` file:

```powershell
cargo build --release
```

The compiled release executable will be located at:
```
target/release/pocket-cli.exe
```

Copy this executable to your target thumb drive or environment, and your zero-dependency edge AI portal is ready for offline execution!
