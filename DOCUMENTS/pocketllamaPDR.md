Preliminary Design Review (PDR): Pocket LLaMA
Project Name: Pocket LLaMA
Document Type: Project Definition & Architecture Blueprint
Target Environment: Disconnected Edge Nodes, Tactical Hardware, Air-Gapped Workstations
Core Stack: Rust, Tokio, Axum, HuggingFace Candle, Rust-Embed

1. Executive Summary
Pocket LLaMA is a hyper-optimized, single-binary local Large Language Model (LLM) runner. Designed to execute sub-4B parameter quantized models (e.g., Phi-3, Qwen-1.5B, TinyLlama) entirely offline, it bypasses the need for cloud compute, GPU dependencies (CUDA), or external heavy runtimes like Node.js or Docker.

The resulting artifact is a single executable (.exe or ELF) that serves dual functions: a high-speed interactive terminal (REPL) and a local, OpenAI-compatible API server with an embedded web-based chat interface.

2. System Architecture & Crate Topology
The project will utilize a Virtual Cargo Workspace to strictly decouple domains, mirroring high-reliability systems engineering practices.

Crate Name | Domain | Primary Dependencies | Responsibility
--- | --- | --- | ---
pocket-core | Domain Models & Errors | thiserror, anyhow, serde | Centralized error handling, configuration schemas, and shared type definitions. Enforces the zero-panic policy.
pocket-engine | ML Inference | candle-core, candle-transformers, tokenizers | Loads GGUF weights to CPU. Executes matrix math inside tokio::task::spawn_blocking. Manages context windows.
pocket-server | API Gateway | axum, tokio, serde_json | Exposes /v1/chat/completions OpenAI-compatible REST endpoints. Manages SSE token streaming to clients.
pocket-ui | Frontend Assets | rust-embed | Compiles vanilla HTML/CSS/JS dark-mode chat interface directly into the binary's RAM allocation.
pocket-cli | Controller / REPL | clap, rustyline, tokio | The binary entry point. Parses arguments, manages the interactive terminal loop, and orchestrates the workspace crates.

3. Core Technical Directives
To ensure deployment resilience in constrained or zero-trust environments, the following architectural strictures apply:
- The Single-Binary Appliance Pattern: The final product must require zero external dependencies at runtime. All web assets, CSS, and routing logic must be baked into the compiled Rust binary.
- Zero-Panic Error Propagation: Explicit prohibition of .unwrap() and .expect() outside of initialization. All thread boundaries and IO operations must return and handle Result<T, PocketError>.
- Asynchronous Isolation: LLM inference is fundamentally a synchronous, CPU-blocking operation. To prevent starvation of the Tokio reactor, all tensor math and token generation loops must be isolated inside dedicated background threads using tokio::task::spawn_blocking.
- Deterministic Token Streaming: The UI and API must not wait for full response generation. Output must stream dynamically via mpsc channels and Server-Sent Events (SSE) to provide real-time telemetry/typing effects.

4. Implementation Phasing
Phase 1: Workspace Scaffolding & Terminal REPL
- Objective: Establish the foundation and build a continuous interactive terminal loop.
- Deliverables: Scaffold the workspace. Implement rustyline REPL in pocket-cli.
- Validation: The CLI accepts continuous user input, simulates a processing delay, prints a mock response, and safely returns to the prompt without leaking memory.

Phase 2: The Inference Engine & Token Streaming
- Objective: Integrate HuggingFace candle and stream raw LLM output to the terminal.
- Deliverables: Build pocket-engine. Load a target GGUF file to Device::Cpu.
- Validation: Implement a LogitsProcessor streamer. As each token is generated in the background thread, it is flushed to stdout via an mpsc channel, rendering a real-time typing effect.

Phase 3: Context & Memory Management
- Objective: Implement conversation history and template formatting.
- Deliverables: Build a ConversationManager struct. Maintain a rolling Vec<Message>.
- Validation: The engine successfully formats multi-turn histories using the specific chat template of the loaded model (e.g., <|user|>{text}<|end|>\n<|assistant|>) prior to context window ingestion.

Phase 4: The Local API Server (OpenAI Compatible)
- Objective: Expose the engine to external local applications.
- Deliverables: Build pocket-server using axum. Bind to 127.0.0.1:8080.
- Validation: A standard cURL request formatted to OpenAI API specs directed at /v1/chat/completions successfully returns a streamed or static JSON response from the local Rust engine.

Phase 5: The Embedded Chat HUD
- Objective: Bake a graphical user interface into the binary.
- Deliverables: Build pocket-ui using rust-embed. Create a vanilla HTML/CSS/JS frontend.
- Validation: Running the executable automatically serves a ChatGPT-style web interface that connects seamlessly to the pocket-server backend.

5. Development Launch Protocol
To initiate the project, provision a new repository directory and execute the following system directive to the AI coding agent:

System Directive: Initiate Project "Pocket LLaMA" - Phase 1
We are building a high-performance, single-binary local LLM runner in Rust. It will execute sub-4B parameter GGUF models on the CPU using Hugging Face candle.

Your Task: Execute Phase 1 (Workspace & REPL)
1. Initialize a Cargo virtual workspace.
2. Create pocket-core to house our thiserror definitions.
3. Create pocket-cli using clap for command-line arguments.
4. In pocket-cli, implement an asynchronous interactive chat loop (REPL) using the rustyline crate.
5. The REPL should accept user input, print a simulated "thinking..." delay, print a mock response, and return to the prompt.
6. Ensure the workspace complies with cargo clippy -- -D warnings and cargo fmt.
7. Output confirmation once the workspace is scaffolded and the mock chat loop compiles.
