import paramiko


def ssh_execute(
    host: str,
    port: int,
    username: str,
    password: str,
    command: str,
    timeout: int = 30,
    get_pty: bool = False,
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
        output = stdout.read().decode("utf-8")
        error = stderr.read().decode("utf-8")

        if error:
            raise Exception(f"Error executing command: {error}")

        return output
    finally:
        client.close()
