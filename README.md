# sanbox

> A sandbox by agents, for agents.

https://github.com/user-attachments/assets/35238607-b5a3-4bab-b834-ae428f0b70c0

## Live Demo

- <https://chenhunghan.github.io/sanbox/>

## CLI

The binary name is `san`.

Run it with a profile word and an optional extra word:

```bash
san claude box
san codex box
san openclaw box
```

## Install

### Download a prebuilt binary

<https://github.com/chenhunghan/sanbox/releases>

## GitHub Pages

The website is deployed from GitHub Actions to:

https://chenhunghan.github.io/sanbox/

The workflow builds the wasm bundle into `docs/` and publishes that directory to GitHub Pages.

## Threat Isolation and Security Policy

Based on the public docs for the current local-agent sandbox landscape:

| Solution | Default local isolation boundary | Threat isolation | Security policy model | Notes |
| --- | --- | --- | --- | --- |
| [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/architecture/) | Per-sandbox microVM with a private Docker daemon | Strongest turnkey laptop-local boundary in public docs; separate kernel, no host Docker access, no cross-sandbox reachability | Host-side network policy and credential injection through Docker's proxy model | Best fit when an agent needs real Docker access but should stay out of the host |
| [NVIDIA OpenShell](https://github.com/NVIDIA/OpenShell) | Isolated container sandbox with policy-enforced egress | Strong on containment and governance, but the public docs emphasize policy more than a hypervisor boundary | Deny-by-default YAML policy across filesystem, network, process, and inference, with hot-reload for dynamic policy sections | Best fit when least-privilege controls matter more than raw VM-style separation |
| [Alibaba OpenSandbox](https://github.com/alibaba/OpenSandbox) | Docker locally, Kubernetes in larger deployments | Medium by default, high when paired with hardened runtimes such as gVisor, Kata, or Firecracker | Unified ingress and egress controls with runtime-level access control and resource quotas | Broadest platform: coding agents, browser sandboxes, remote dev, code execution, RL |
| [kubernetes-sigs/agent-sandbox](https://github.com/kubernetes-sigs/agent-sandbox) | Kubernetes `Sandbox` CRD backed by a pod/runtime | Runtime-dependent; stronger when paired with gVisor or Kata | Mostly delegated to the chosen runtime and cluster policy stack | Best fit for long-lived, stateful, singleton agent runtimes on Kubernetes |
| [AgentScope Runtime](https://runtime.agentscope.io/en/sandbox/sandbox.html) | Docker by default, optional gVisor or BoxLite | Medium by default, stronger when using hardened local runtimes | SDK and sandbox-server controls, but less policy-granular than OpenShell | Good SDK-first choice for tool, browser, filesystem, mobile, and training sandboxes |

`OpenClaw` is not listed as a separate isolation primitive because its own sandbox mode delegates to backends like Docker, SSH, or OpenShell. In practice, its security posture depends on the backend you select.
