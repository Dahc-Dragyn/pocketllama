Software Design Document (SDD)
Project Name: Pocket LLaMA
Version: 1.0 Draft
Deployment Target: Tactical Thumb Drive (Single Binary, Air-Gapped)

1. Architectural Overview
Pocket LLaMA is designed as a Monolithic Single-Binary Appliance. To achieve high maintainability, the internal codebase is decoupled into a Virtual Cargo Workspace containing five distinct crates, all compiled down to a single executable.
The system relies on strict concurrency isolation: highly concurrent, non-blocking asynchronous I/O (handled by Tokio) must be completely separated from synchronous, blocking, CPU-bound matrix mathematics (handled by Candle).

2. Workspace Subsystem Design
2.1 pocket-core (Shared Primitives & Safety)
Responsibility: Houses global error definitions, configuration structs, and the Pre-Flight Hardware Guardrail.
The Guardrail Implementation:
- Utilizes the sysinfo crate to poll System::total_memory().
- Utilizes std::fs::metadata() to calculate the byte size of the target .gguf file.
- Ensures file_size_bytes < (total_memory_bytes * 0.8).
- If the threshold is breached, execution halts with a safe PocketError::MemoryInsufficient to prevent OS-level disk swapping.

2.2 pocket-engine (Machine Learning Pipeline)
Responsibility: Manages model weight ingestion, tokenization, and inference generation.
Memory Management: Models are strictly loaded into candle_core::Device::Cpu.
Execution Boundary: The core generation loop (autoregressive token sampling) is a blocking operation. It must be wrapped in a closure and passed to tokio::task::spawn_blocking.
The Streaming Pipeline: Inside the blocking thread, a LogitsProcessor samples the next token. This token is decoded and immediately sent across an asynchronous boundary using a tokio::sync::mpsc (Multi-Producer, Single-Consumer) channel to the active UI or API subscriber.

2.3 pocket-server (API Gateway)
Responsibility: An Axum-based HTTP router mapping internal engine events to standard network protocols.
Endpoints:
- POST /v1/chat/completions: Accepts standard OpenAI-formatted JSON payloads. Parses the payload, feeds the system/user context to pocket-engine, and returns either a static JSON response or a chunked Server-Sent Events (SSE) stream based on the stream: true flag.

2.4 pocket-ui (Embedded Web Dashboard)
Responsibility: Provides the visual graphical user interface.
Implementation: A vanilla HTML/CSS/JS single-page application.
Embedding: Uses the rust-embed macro pointing to the /assets directory. The Axum router serves these files directly from RAM upon startup. No file I/O is required to serve the frontend.

2.5 pocket-cli (Controller & REPL)
Responsibility: The binary entry point. Parses launch arguments and orchestrates the workspace.
The REPL Mode: If launched in terminal mode, it instantiates rustyline for a Read-Eval-Print Loop. It captures stdin, passes it to pocket-engine, and loops stdout flushing as tokens arrive via the mpsc channel to simulate real-time typing.

3. Data Flow & Concurrency Map
The most critical architectural hazard in this system is blocking the async reactor. The data flow below illustrates how input crosses from the async world into the sync world and back safely.

Step 1: Ingestion (Async)
User submits a prompt (via CLI or Web API). The pocket-cli or pocket-server receives this async event.

Step 2: Context Formatting (Async)
The prompt is appended to the ConversationManager history. The entire history is formatted into a single string using the model's specific chat template.

Step 3: The Boundary Crossing (Sync Handoff)
The formatted string and an mpsc::Sender clone are moved into tokio::task::spawn_blocking. The async reactor is now free to handle other web requests.

Step 4: Inference & Streaming (Sync -> Async)
Inside the blocking thread, Candle tokenizes the string and begins matrix multiplication.
Loop:
1. Candle generates Token A.
2. Thread calls sender.blocking_send(Token A).
3. The async reactor wakes up the receiver (CLI or Web API) and flushes Token A to the screen/network.
4. Candle generates Token B. (Repeat).

Step 5: Termination (Async)
Upon reaching an EOS (End of Sequence) token, the blocking thread returns safely, dropping the sender. The receiver detects the channel closure and finalizes the network request or REPL loop.

4. Error Handling (Zero-Panic Policy)
- The system shall implement thiserror for library crates (pocket-engine, pocket-core) to provide explicit, matchable error variants (e.g., TensorError, TokenizerError, HardwareError).
- The system shall implement anyhow in application crates (pocket-cli, pocket-server) for rich context chaining (e.g., .context("Failed to bind Axum server to port 8080")).
- Explicit .unwrap() calls are forbidden. All initialization sequences must gracefully bubble errors to main(), returning an exit code of 1 and printing a human-readable diagnostic.

With the Blueprint, the SRS, and this SDD, you have a complete technical specification package for Pocket LLaMA. All of the architectural ambiguities regarding how the CPU math interacts with the web server have been solved on paper.
