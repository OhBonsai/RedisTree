import pytest
import redis
import time


@pytest.fixture(scope="session")
def redis_client():
    client = redis.Redis(host="127.0.0.1", port=6379, decode_responses=True)
    client.ping()
    return client


@pytest.fixture(scope="function", autouse=True)
def clear_db(redis_client):
    redis_client.flushdb()