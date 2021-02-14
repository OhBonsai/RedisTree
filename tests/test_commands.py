import pytest
from redis.exceptions import ResponseError


def test_init(redis_client):
    # wrong format tree string will raise ResponseError
    with pytest.raises(ResponseError, match="() is not closed or no root"):
        redis_client.execute_command("tree.init", "hello", "0 (1(2")

    assert redis_client.execute_command("tree.get", "hello") is None
    redis_client.execute_command("tree.init", "hello", "0 (1 2)")


def test_get(redis_client):
    redis_client.execute_command("tree.init", "hello", "0 (1 2 (a b (d)) e f (g h))")
    assert redis_client.execute_command("tree.get", "hello") == "0( 1 2( a b( d ) ) e f( g h ) )"


def test_del(redis_client):
    redis_client.execute_command("tree.init", "hello", "0 (1 2 (a b (d)) e f (g h))")
    redis_client.execute_command("tree.del", "hello")
    assert redis_client.execute_command("tree.get", "hello") is None


def test_get_subtree(redis_client):
    redis_client.execute_command("tree.init", "hello", "0 (1 2 (a b (d)) e f (g h))")
    assert redis_client.execute_command("tree.get_subtree", "hello", "f") == "f( g h )"


def test_set_subtree(redis_client):
    redis_client.execute_command("tree.init", "hello", "0 (1 2)")
    redis_client.execute_command("tree.set_subtree", "hello", "2", "3 ( 4 5)")
    assert redis_client.execute_command("tree.get", "hello") == "0( 1 2( 3( 4 5 ) ) )"


def test_get_ancestors(redis_client):
    redis_client.execute_command("tree.init", "hello", "0 (1 2 (a b (d)) e f (g h))")
    assert redis_client.execute_command("tree.get_ancestors", "hello", "d") == ["b", "2", "0"]


def test_get_descendants(redis_client):
    redis_client.execute_command("tree.init", "hello", "0 (1 2 (a (k (j) bb) b (d)) e f (g h))")
    assert redis_client.execute_command("tree.get_descendants", "hello", "2") == ["2", "a", "b", "k", "bb", "d", "j"]


def test_get_father(redis_client):
    redis_client.execute_command("tree.init", "hello", "0 (1 2 (a (k (j) bb) b (d)) e f (g h))")
    assert redis_client.execute_command("tree.get_father", "hello", "j") == "k"


def test_get_children(redis_client):
    redis_client.execute_command("tree.init", "hello", "0 (1 2 (a (k (j) bb) b (d)) e f (g h))")
    assert redis_client.execute_command("tree.get_children", "hello", "0") == ["1", "2", "e", "f"]
