import os
import json


def execute_commands_file(file_name, redis_client):
    cwd = os.getcwd()
    file_name = os.path.join(cwd, "./tests/files", file_name)

    for line in open(file_name, "r").readlines():
        if line.strip().startswith("#"):
            continue

        args = json.loads(line)
        redis_client.execute_command(*args)


if __name__ == '__main__':
    import pytest
    pytest.main(["test_commands.py::test_get_descendants"])
    # pytest.main()
