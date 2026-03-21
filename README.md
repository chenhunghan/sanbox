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

## Comparison

Based on the public docs for the current local-agent sandbox landscape:


| Solution | Art | Container default | MicroVM default | Hardened runtime option | FS policy | Network policy | Process policy | Creds kept outside sandbox |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **sanbox** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/architecture/) | ❌ | ❌ | ✅ | ✅ | ⚠️ | ✅ | ❌ | ✅ |
| [NVIDIA OpenShell](https://github.com/NVIDIA/OpenShell) | ❌ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| [Alibaba OpenSandbox](https://github.com/alibaba/OpenSandbox) | ❌ | ✅ | ❌ | ✅ | ⚠️ | ✅ | ⚠️ | ⚠️ |
| [kubernetes-sigs/agent-sandbox](https://github.com/kubernetes-sigs/agent-sandbox) | ❌ | ✅ | ❌ | ✅ | ⚠️ | ⚠️ | ⚠️ | ❌ |
| [AgentScope Runtime](https://runtime.agentscope.io/en/sandbox/sandbox.html) | ❌ | ✅ | ❌ | ✅ | ⚠️ | ⚠️ | ❌ | ❌ |

Legend: `✅` yes, `❌` no, `⚠️` optional or runtime-dependent

For `sanbox`, the row reflects implemented isolation and policy primitives. This project is a visual CLI/web animation, not a real execution sandbox.
