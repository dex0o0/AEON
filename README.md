# AEON Daemon

<br/>

## Project Overview

AEON is a system daemon designed for **comprehensive server and system monitoring**. This service continuously tracks key system resources to ensure optimal performance and provide insights into your server's health.

<hr/>

## Monitored Resources

AEON actively monitors the following critical system metrics:

*   **Swap Space Usage**: Tracks memory swapping activity.
*   **CPU Usage**: Monitors overall processor load.
*   **GPU Usage**: Observes graphics processing unit utilization.
*   **Network Connections**: Analyzes network traffic and connection status.
*   **Disks Usage**:check disk Usage
*   **Backup**:system backup make
## How to Collaborate

We welcome contributions from anyone interested in improving AEON! Whether you're a seasoned Rust developer, a Linux enthusiast, or just looking to learn, there's a place for you here.

**First step: Clone the project**

To get started, clone the repository to your local machine using the following command:

```bash
git clone https://gitlab.chabokan.net/dex0o0/aeon.git
```
**Next steps:**

Once you have the project cloned, here’s how you can get involved:

Explore the Codebase:

Familiarize yourself with the project structure and the current implementation. Pay attention to the src/ directory where the main logic resides.
Check the Issues:

Visit the Issues section on our GitLab page: [Link to your project's issues page on GitLab]
You'll find a list of tasks, feature requests, and bug reports. Feel free to pick an issue that interests you!
If you find a bug or have a feature idea, please open a new issue to discuss it.
Set Up Your Development Environment:

Ensure you have Rust and Cargo installed. If not, the easiest way is via rustup:
bash

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
You'll also need bash and systemd (which are standard on Linux). Make sure you have make or cargo installed for building.
Start Contributing:

Pick an Issue: Choose an issue from the GitLab issues page that you'd like to work on. If you're new, look for issues tagged with good first issue or help wanted.
Create a New Branch: Before making any changes, create a new branch for your work. This keeps your changes separate and organized.
bash

```bash
git checkout -b your-branch-name
```
(Replace your-branch-name with something descriptive, like fix-network-bug or add-disk-monitoring)
Make Your Changes: Write your code, add new features, or fix bugs.
Test Your Changes: Make sure your changes work correctly and don't introduce new problems. If possible, add tests for your changes.
Commit Your Changes: Commit your changes with clear and descriptive messages.
bash

```bash
git add .
git commit -m "Add descriptive commit message for your changes"
```
Push Your Branch: Push your branch to your fork or directly to the repository (if you have permissions).
bash

```bash
git push origin your-branch-name
```
**Submit a Merge Request (MR)**:

Go to the AEON GitLab repository page.
You should see an option to create a Merge Request from your recently pushed branch.
Describe your changes clearly in the MR description. Explain what the MR does and why.
Link any related issues.
Communication:

Questions & Discussion: If you have questions or want to discuss ideas, please use the Issues section on GitLab 
Code Reviews: We aim to provide constructive feedback on all Merge Requests. Please be open to suggestions.
