# Agent RedTeam

**"The first framework that treats AI agents like untrusted binaries — because that's exactly what they are."**

Autonomous adversarial tester for AI coding agents. Engineered for security research, robustness validation, and automated vulnerability registration.

## Features
- **Neuroevolution**: Neural networks that evolve attack strategies.
- **Auto-CVE Registration**: Automated documentation and classification of findings.
- **Multi-Agent Testing**: Support for Claude, OpenAI, Gemini, and Groq.
- **Parallel Execution**: Engineered for high-throughput testing with Rayon.
- **Adversarial Vectors**: Extensive library including Prompt Injection, Tool Poisoning, and API Key Phishing.

## Quick Start
```bash
./install.sh
agent-redteam benchmark
agent-redteam report --format html --output report.html
```

## License
MIT
