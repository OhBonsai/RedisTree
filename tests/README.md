# System tests for redis tree
This folder contains the system tests for RedisTree. System tests are
written in Python and Pytest framework

## Running
All test run in docker, So you need Docker installed. Then you can prepare
the setup adn run all teh tests with:
```
make test
```

Running a single test, e.g.:
```
make test case="xab/test"
```