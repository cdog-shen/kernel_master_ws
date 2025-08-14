from pathlib import Path
from typing import Union
import time
import paramiko


def ssh_execute(
    host: str,
    port: int,
    username: str,
    password: str,
    command: str,
    timeout: int = 30,
    get_pty: bool = False,
    encoding: str = "utf-8",
    sleep_time=0.2,
) -> str:
    """
    Execute a command on a remote server via SSH.

    :param host: Remote server hostname or IP address
    :param port: SSH port (default is usually 22)
    :param username: SSH username
    :param password: SSH password
    :param command: Command to execute on the remote server
    :return: Output of the command execution
    """
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())

    try:
        client.connect(hostname=host, port=port, username=username, password=password)
        stdin, stdout, stderr = client.exec_command(
            command, timeout=timeout, get_pty=get_pty
        )
        output = stdout.read().decode(encoding)
        error = stderr.read().decode(encoding)

        if error:
            raise Exception(f"Error executing command: {error}")

        return output
    finally:
        client.close()
        time.sleep(sleep_time)


def sftp_download(
    host: str,
    port: int,
    username: str,
    password: str,
    remote_path: Union[str, Path],
    local_path: Union[str, Path],
    timeout: int = 30,
    encoding: str = "utf-8",
    sleep_time=0.2,
) -> None:
    """
    Recursively download a file or directory via SFTP.

    :param host: Remote server hostname or IP address
    :param port: SSH port
    :param username: SSH username
    :param password: SSH password
    :param remote_path: Remote file/dir to download
    :param local_path: Local destination directory
    :param timeout: Socket timeout (seconds)
    :param encoding: Filename encoding
    """
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())

    try:
        client.connect(
            hostname=host,
            port=port,
            username=username,
            password=password,
            timeout=timeout,
        )
        sftp = client.open_sftp()

        remote_path = Path(remote_path)
        local_path = Path(local_path)

        def _download(r_path: Path, l_path: Path) -> None:
            try:
                attr = sftp.stat(str(r_path))
            except FileNotFoundError:
                raise FileNotFoundError(f"{r_path} does not exist on remote")

            if attr.st_mode & 0o40000:  # directory
                l_path.mkdir(parents=True, exist_ok=True)
                for fname in sftp.listdir_attr(str(r_path)):
                    _download(r_path / fname.filename, l_path / fname.filename)
            else:  # file
                l_path.parent.mkdir(parents=True, exist_ok=True)
                sftp.get(str(r_path), str(l_path))

        _download(remote_path, local_path / remote_path.name)

    finally:
        client.close()
        time.sleep(sleep_time)


def sftp_upload(
    host: str,
    port: int,
    username: str,
    password: str,
    local_path: Union[str, Path],
    remote_path: Union[str, Path],
    timeout: int = 30,
    sleep_time=0.2,
) -> None:
    """
    Recursively upload a file or directory via SFTP.

    :param host: Remote server hostname or IP address
    :param port: SSH port
    :param username: SSH username
    :param password: SSH password
    :param local_path: Local file/dir to upload
    :param remote_path: Remote destination path (directory or full file path)
    :param timeout: Socket timeout (seconds)
    """
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())

    try:
        client.connect(
            hostname=host,
            port=port,
            username=username,
            password=password,
            timeout=timeout,
        )
        sftp = client.open_sftp()

        local_path = Path(local_path)
        remote_path = Path(remote_path)

        def _upload(l_path: Path, r_path: Path) -> None:
            if l_path.is_dir():
                # 确保远端目录存在
                try:
                    sftp.listdir(str(r_path))  # 触发异常检测
                except FileNotFoundError:
                    sftp.mkdir(str(r_path))
                for child in l_path.iterdir():
                    _upload(child, r_path / child.name)
            else:
                # 确保远端父目录存在
                try:
                    sftp.listdir(str(r_path.parent))
                except FileNotFoundError:
                    # 递归创建远端父目录
                    parent = r_path.parent
                    if parent:
                        client.exec_command(f"mkdir -p {parent}")[1].read()
                sftp.put(str(l_path), str(r_path))

        # 如果 remote_path 以 / 结尾或已存在目录，则视为目录
        try:
            sftp.listdir(str(remote_path))
            _upload(local_path, remote_path / local_path.name)
        except FileNotFoundError:
            _upload(local_path, remote_path)

    finally:
        client.close()
        time.sleep(sleep_time)