Software Requirements Specification (SRS)
Project Name: Pocket LLaMA
Version: 1.0 Draft
Target Environment: Disconnected Edge Nodes, Tactical Hardware, Air-Gapped Workstations

1. Introduction
1.1 Purpose: The purpose of this document is to define the functional and non-functional requirements for "Pocket LLaMA," a single-binary, zero-dependency Rust application designed to run quantized Large Language Models (LLMs) locally on standard CPU hardware.

1.2 Scope:
Pocket LLaMA will serve as a local AI appliance. It will ingest sub-4B parameter .gguf files and provide three distinct interfaces: an interactive terminal REPL, an OpenAI-compatible REST API, and an embedded graphical web dashboard.

2. Overall Description
2.1 User Characteristics: Users range from software engineers to tactical field operators. The system must assume the user does not have a background in machine learning. Error messages must be highly actionable in plain English.

2.2 Operating Environment:

OS: Windows, macOS, Linux (Cross-compiled standard binaries).

Hardware: Standard consumer or tactical laptops (minimum 4GB RAM, dual-core CPU). No dedicated GPU (CUDA/Metal) is required.

Network: Fully offline/air-gapped.

3. System Features & Functional Requirements
3.1 The Pre-Flight Hardware Guardrail (Safety Interlock)
Description: The system must prevent catastrophic out-of-memory (OOM) OS freezes caused by loading models too large for the physical hardware.

Req 3.1.1: On startup, the system shall read the physical file size of the target .gguf model.

Req 3.1.2: The system shall query the host OS for total available physical RAM.

Req 3.1.3: If the model file size exceeds 80% of the host's total physical RAM, the system shall abort initialization and throw a fatal PocketError::MemoryInsufficient warning to the console.

Req 3.1.4: The system shall provide an override flag (--force-ram) to bypass this check for power users.

3.2 The Inference Engine
Description: The core ML tensor execution environment.

Req 3.2.1: The engine shall load .gguf quantized model weights strictly onto Device::Cpu.

Req 3.2.2: The engine shall execute all tensor mathematical operations inside isolated tokio::task::spawn_blocking background threads to prevent async runtime starvation.

Req 3.2.3: The engine shall stream generated tokens sequentially via channels rather than waiting for the entire response to complete.

3.3 Context & Conversation Management
Description: The system's ability to "remember" the current conversation.

Req 3.3.1: The system shall maintain a rolling sequence of User and Assistant messages.

Req 3.3.2: The system shall automatically inject the correct Chat Template (e.g., ChatML, Llama-3 format) based on the model's tokenizer metadata before inference.

3.4 API & UI Interfaces
Description: How the user interacts with the engine.

Req 3.4.1: The system shall provide a terminal REPL utilizing rustyline for raw CLI interaction.

Req 3.4.2: The system shall host an embedded Axum web server exposing POST /v1/chat/completions compliant with standard OpenAI schema.

Req 3.4.3: The system shall embed a vanilla HTML/CSS/JS web dashboard compiled directly into the binary using rust-embed.

4. Non-Functional Requirements
4.1 Reliability (Zero-Panic Policy):
The application shall not crash unexpectedly. The use of .unwrap() or .expect() is strictly forbidden outside of static initialization. All runtime errors must be gracefully caught and propagated using the anyhow and thiserror crates.

4.2 Portability (Single-Binary):
The final compiled artifact must not require the installation of external runtimes (Node.js, Docker, Python).

4.3 Performance (Concurrency):
The asynchronous network and UI server must remain highly responsive (under 50ms latency) even while the CPU is fully saturated by the inference engine running in the background thread.

How We Build the Guardrail (For Antigravity)
When we get to building that guardrail, it is actually a very clean Rust implementation. We will add the sysinfo crate to our pocket-core dependencies.

The logic will look roughly like this:

Use std::fs::metadata(model_path)?.len() to get the exact byte size of the GGUF file.

Use sysinfo::System::new_all().total_memory() to get the physical RAM of the laptop.

Compare the two. If the file is 40GB and the RAM is 8GB, we instantly print: "CRITICAL: Model size (40GB) exceeds physical RAM (8GB). System would freeze. Use --force-ram to bypass."
