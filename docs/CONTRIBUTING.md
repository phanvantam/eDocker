# Contributing to eDocker 🛠️

First off, thank you for considering contributing to eDocker! It's people like you who make eDocker a better tool for everyone.

This guide will help you get your development environment set up and walk you through the process of making your first contribution.

## Prerequisites

To build eDocker, you need the following tools installed on your machine:

- **Node.js** (v18 or higher)
- **Rust** (Stable toolchain via [rustup](https://rustup.rs/))
- **Yarn** (Package manager)
- **Docker** (Running on your local machine)
- **C/C++ Build Tools** (Required for Tauri):
  - **macOS:** Xcode Command Line Tools (`xcode-select --install`)
  - **Windows:** Visual Studio Build Tools
  - **Linux:** `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`

## Local Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/phanvantam/eDocker.git
   cd eDocker
   ```

2. **Install frontend dependencies:**
   ```bash
   yarn install
   ```

3. **Run the application in development mode:**
   ```bash
   yarn tauri dev
   ```
   *This will start the Vite dev server and compile the Rust backend. The first build may take a few minutes.*

## Project Structure

- `src/`: Vue 3 frontend code.
- `src-tauri/`: Rust backend code and Tauri configuration.
- `docs/`: Documentation and assets.

## Workflow

1. **Find an issue** or propose a new feature.
2. **Create a new branch** for your work: `git checkout -b feat/your-feature-name`.
3. **Write code** and ensure it follows the project's style.
4. **Test your changes** thoroughly.
5. **Commit your changes**: We follow conventional commits (e.g., `feat: add terminal support`).
6. **Push to your fork** and submit a **Pull Request**.

## Need Help?

If you have questions, feel free to open a GitHub Issue or reach out to the maintainers.

---

*Let's build the best Docker GUI together!*

[Back to Main README](../README.md)
