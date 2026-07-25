# Security policy

Ogunedo is pre-audit software. Do not use the included parameter sets to protect real value.

Requester wallet material is out of scope for Git. Keep `.env` and `.env.*` local and untracked. The repository intentionally tracks only `.env.example`, which contains variable names and empty values.

The network proving commands read `NETWORK_PRIVATE_KEY` only inside the local requester process. Do not paste keys into issues, pull requests, logs, CI secrets, or benchmark reports.

Report vulnerabilities privately to **josephnedo007@gmail.com** with:

- affected version or commit;
- threat model and impact;
- reproduction steps or proof of concept;
- suggested remediation when available.

Please do not open a public issue for an unpatched vulnerability. Acknowledgement and disclosure timing will be coordinated with the reporter.
