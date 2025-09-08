import os
import shutil
import tempfile
import ansible_runner
from pathlib import Path
from typing import Dict, Any, Optional, List, Sequence, cast

# const
DEFAULT_TIMEOUT: int = 300
PRIVATE_DATA_ROOT = Path("/tmp/ansible_agent")
PLAYBOOK_DIR = PRIVATE_DATA_ROOT / "project"
INVENTORY_DIR = PRIVATE_DATA_ROOT / "inventory"

PLAYBOOK_DIR.mkdir(parents=True, exist_ok=True)
INVENTORY_DIR.mkdir(parents=True, exist_ok=True)


# privite
def _prepare_private_data_dir(name: str) -> Path:
    tmp = Path(tempfile.mkdtemp(prefix=f"ansible_{name}_", dir=PRIVATE_DATA_ROOT))
    (tmp / "project").mkdir()
    (tmp / "inventory").mkdir()
    return tmp


def _write_inventory(hosts: Sequence[str], port: int, tmpdir: Path) -> Path:
    inv_file = tmpdir / "inventory" / "hosts.ini"
    with inv_file.open("w") as f:
        f.write("[all]\n")
        for h in hosts:
            f.write(f"{h} ansible_port={port}\n")
    return inv_file


def _write_playbook(yml_content: str, tmpdir: Path) -> Path:
    pb_file = tmpdir / "project" / "pb.yml"
    pb_file.write_text(yml_content)
    return pb_file


def _run(
    playbook: Path,
    inventory_path: Path,
    extravars: Optional[Dict[str, Any]] = None,
    passwords: Optional[Dict[str, str]] = None,
    timeout: int = DEFAULT_TIMEOUT,
) -> ansible_runner.Runner:
    return cast(
        ansible_runner.Runner,
        ansible_runner.run(
            private_data_dir=str(inventory_path.parent),
            playbook=str(playbook),
            inventory=str(inventory_path),
            extravars=extravars or {},
            passwords=passwords or {},
            quiet=False,
            cancel_callback=lambda _: False,
            finished_callback=lambda _: None,
            timeout=timeout,
        ),
    )


# public
def ansible_ping(
    hosts: Sequence[str], user: str, password: str, port: int = 22, timeout: int = 60
) -> Dict[str, Any]:
    tmpdir = _prepare_private_data_dir("ping")
    try:
        inv = _write_inventory(hosts, port, tmpdir)
        pb = _write_playbook(
            """---
            - hosts: all
                gather_facts: no
                tasks:
                - ping:
                """,
            tmpdir,
        )
        r = _run(
            pb,
            inv,
            extravars={"ansible_user": user},
            passwords={"conn_pass": password},
            timeout=timeout,
        )

        unreachable = [
            event["stdout"].strip()
            for event in r.events
            if event.get("event") == "runner_on_unreachable"
        ]
        ok = [
            event["stdout"].strip()
            for event in r.events
            if event.get("event") == "runner_on_ok"
        ]
        return {"status": r.status, "unreachable": unreachable, "ok": ok}
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def ansible_shell(
    hosts: Sequence[str],
    user: str,
    password: str,
    command: str,
    port: int = 22,
    timeout: int = DEFAULT_TIMEOUT,
    become: bool = False,  # Become privilege escalation
) -> Dict[str, Any]:
    tmpdir = _prepare_private_data_dir("shell")
    try:
        inv = _write_inventory(hosts, port, tmpdir)
        pb = _write_playbook(
            f"""---
            - hosts: all
                gather_facts: no
                tasks:
                - name: run shell
                    shell: {command}
                    become: {'yes' if become else 'no'}
            """,
            tmpdir,
        )
        r = _run(
            pb,
            inv,
            extravars={"ansible_user": user},
            passwords={"conn_pass": password},
            timeout=timeout,
        )

        stdout = [ev["stdout"] for ev in r.events if ev.get("event") == "runner_on_ok"]
        stderr = [
            ev["stderr"] for ev in r.events if ev.get("event") == "runner_on_failed"
        ]
        return {"status": r.status, "rc": r.rc, "stdout": stdout, "stderr": stderr}
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)


def ansible_playbook(
    playbook_content: str,  # Playbook content
    hosts: Sequence[str],  # Target hosts
    user: str,  # SSH user
    password: str,  # SSH password
    port: int = 22,  # SSH port
    extravars: Optional[Dict[str, Any]] = None,  # Extra variables
    timeout: int = DEFAULT_TIMEOUT,  # Timeout
) -> Dict[str, Any]:
    tmpdir = _prepare_private_data_dir("pb")
    try:
        inv = _write_inventory(hosts, port, tmpdir)
        pb = _write_playbook(playbook_content, tmpdir)
        r = _run(
            pb,
            inv,
            extravars={**(extravars or {}), "ansible_user": user},
            passwords={"conn_pass": password},
            timeout=timeout,
        )
        stats = next(
            (
                ev["event_data"]
                for ev in r.events
                if ev.get("event") == "playbook_on_stats"
            ),
            {},
        )
        return {"status": r.status, "rc": r.rc, "stats": stats}
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)
