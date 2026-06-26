# AEON.service

AEON is a lightweight, modular system daemon written in **Rust** and **Shell** designed for monitoring Linux server resources (including CPU, Memory, Disk, Network, GPU, and backup tasks).

## Features

- **Modular Monitoring:** Separate modules for CPU, RAM, Disk, Network, and GPU.
- **Resource Efficient:** Low-overhead background daemon that collects system metrics with minimal impact.
- **Backup & Health Checks:** Basic backup monitoring and system health tracking.
- **Easy Installation:** Comes with a quick setup script (`install.AEON.sh`).

## Getting Started

### Prerequisites

- Rust (Cargo)
- Linux Environment

### Installation

To install and run the AEON daemon, use the provided installation script:

```bash
chmod +x install.AEON.sh
./install.AEON.sh
```

## Communication

- Questions & Discussion: If you have questions or want to discuss ideas, please open an Issue on GitHub.
- Code Reviews: We aim to provide constructive feedback on all Pull Requests. Please be open to suggestions and code improvements.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
