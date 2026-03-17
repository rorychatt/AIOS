# AIOS (AI-First Operating System)

> The Next-Generation Operating System built from the ground up for the age of Artificial Intelligence.

![AIOS Concept](https://img.shields.io/badge/Status-Ideation-blue) ![License](https://img.shields.io/badge/License-MIT-green)

---

## 🌌 The Vision

Traditional operating systems like Windows and macOS were built for an era where humans were the primary, and often only, consumers of computing resources. Their architecture is fundamentally tied to human-centric paradigms: graphical desktop environments, overlapping windows, mouse cursors, and visual feedback loops.

**In the age of AI, this paradigm is obsolete.**

When an AI agent interacts with a computer, it doesn't need a 4K display, window managers, or complex UI rendering. It needs fast, reliable, and deeply integrated APIs.

**AIOS** is a radical reimagining of the operating system. We propose an OS where every design choice answers one question: _"What is best for AI?"_

## 🧠 Core Philosophy

### 1. API-First, Always

Instead of graphical applications that are difficult for agents to parse, AIOS applications are built as APIs from day one. If an app exists on the OS, it surfaces its entire capability set via an accessible, system-level API and predictable CLI.

### 2. The Next Generation of MCP

Model Context Protocol (MCP) is currently used to bridge specific tools with LLMs. In AIOS, **the OS itself is the ultimate MCP server**. Every file, system setting, active process, and installed application is intrinsically exposed as context that natively understands how to talk to LLMs.

### 3. Agent-Native User Experience

Humans interact with AIOS not by clicking through layers of settings, but by collaborating with the OS's native AI layer. You state your intent; the AI layer calls the underlying OS APIs to execute it instantly.

### 4. Zero "Window / macOS BS"

No bloated window managers. No legacy graphics stacks consuming system resources just to idle on a desktop background. The visual layer (if needed at all) is a secondary, lightweight view into a system primarily running headless and agent-driven workflows.

---

## 🏗️ Architecture & Features (Proposed)

- **Universal CLI/API Gateway**: Every installed package registers a programmatic schema. Apps don't have visual frontends; they have conversational and programmatic interfaces.
- **Context-Aware File System**: A file system that indexes files not just by metadata and path, but by semantic meaning, making it instantly searchable by LLMs.
- **Secure Agent Sandboxing**: Built-in permission models dynamically restrict what an AI agent can execute or read, preventing prompt injection or rogue agents from compromising the system kernel.
- **Resource Optimization**: Without the need for massive desktop compositors, CPU and GPU resources are freed up to run local LLM instances.

---

## 🖥️ AIOS Interface (Ivy Dashboard)

While AIOS operates primarily headlessly, it features an ultra-modern, interactive **Ivy Framework Web Dashboard**. 

Built in C# and .NET 10, the `aios-dashboard` acts as a visualization layer running on `http://localhost:5010`. It displays realtime telemetry for the `aios-daemon` and installed plugins, alongside providing a sleek Conversational UI hooked directly into the local Ollama LLM router.

### How to use

Ensure you have Rust and the .NET 10 SDK installed.

```bash
# Clone the repository
git clone https://github.com/yourusername/AIOS.git
cd AIOS

# Start the Operating System
cargo run --bin aios-cli start
```

This single command boots the Daemon in the background, spins up the Ivy Dashboard, and automatically launches your browser into the OS environment.

---

## 🚀 Why AIOS?

Right now, developers spend countless hours writing computer vision scripts, accessibility (A11y) tree parsers, and brittle RPA (Robotic Process Automation) bots just to let an AI click a button on a legacy OS.

By building an OS where apps are fundamentally APIs, we skip the middleman. We move from _“AI looking at a screen”_ to _“AI natively speaking to the machine.”_

---

## 🤝 Contributing

We are at step zero of building the future of computing. If you are passionate about AI agents, operating system design, rust/C, or API architectures, your ideas are needed.

_More contribution guidelines coming soon._

---

_Built for the future. Built for AI._
