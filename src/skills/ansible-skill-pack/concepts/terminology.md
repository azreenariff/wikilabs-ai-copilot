# Ansible Terminology Glossary

## Core Terms

| Term | Definition |
|------|-----------|
| **Playbook** | YAML file defining automation tasks |
| **Play** | A set of tasks to run on a host group |
| **Task** | A single unit of automation |
| **Role** | Reusable playbook component |
| **Module** | Reusable automation unit |
| **Inventory** | Host and group definitions |
| **Handler** | Notification-driven task |
| **Variable** | Dynamic value in playbooks |

## Ansible Core Terms

| Term | Definition |
|------|-----------|
| **ansible** | Ad-hoc command runner |
| **ansible-playbook** | Playbook executor |
| **ansible-doc** | Module documentation |
| **ansible-galaxy** | Collection registry |
| **ansible-vault** | Encryption tool |
| **ansible-inventory** | Inventory management |
| **ansible-config** | Configuration dump |

## Module Terms

| Term | Definition |
|------|-----------|
| **Ansible Module** | Reusable automation code |
| **Builtin Module** | Module included with Ansible |
| **Custom Module** | User-defined module |
| **Module Parameter** | Input option for module |
| **Module Return Value** | Output from module |
| **Module Idempotency** | Safe to run multiple times |

## Collection Terms

| Term | Definition |
|------|-----------|
| **Collection** | Bundled modules, plugins, playbooks |
| **Namespace** | Organization prefix for collections |
| **Ansible Galaxy** | Collection registry |
| **Collection Build** | Packaging collection for distribution |
| **Collection Install** | Installing collection from Galaxy |

## Infrastructure Terms

| Term | Definition |
|------|-----------|
| **Control Node** | Machine running Ansible |
| **Managed Node** | Target host being managed |
| **SSH** | Secure shell transport |
| **WinRM** | Windows remote management |
| **Fork** | Parallel execution count |
| **Become** | Privilege escalation |
| **Privilege Escalation** | Running tasks as different user |

## References

- Ansible Glossary: https://docs.ansible.com/ansible/latest/reference_appendices/glossary.html
- Ansible Core Glossary: https://docs.ansible.com/ansible/latest/