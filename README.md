# RedisTree
RedisTree is a [Redis](https://redis.io/) module that implements Polytree as a native data type. It allows creating,locating,pushing and  detaching tree from Redis keys.
**RedisTree has been running in production environment for one year**


- 哥，来一趟不容易，点个Star呗
- Could you give me a star...
- Donne moi une étoile
- 星をください
- 나에게 별을 줘

## Why?
Store organization data in redis, Need ploytree and some helper function

## Quick Start
```shell
# run redis by docker
docker run -p 6379:6379 -d --name redis-redistree ohbonsai/redistree

# exec redis-cli in redis container
docker exec -it redis-redistree /bin/sh
redis-cli
```

## Commands
- `tree.init key tree_value`
- `tree.get  key`
- `tree.del  key`
- `tree.get_subtree key node_value` 
- `tree.del_subtree key node_value`
- `tree.set_subtree key node_value tree_value`
- `tree.get_ancestors key node_value`
- `tree.get_descendants key node_value`
- `tree.get_father key node_value`
- `tree.get_children key node_value`

### Init Get Del tree from String

```
    a
  /    \
 b      c
 |
 e

127.0.0.1:6379> tree.init hello "a (b (d) c)"
OK
127.0.0.1:6379> tree.get hello
"a( b( d ) c )"
127.0.0.1:6379> tree.del hello
OK
127.0.0.1:6379> tree.get hello
(nil)
127.0.0.1:6379> tree.init hello "a (("
(error) ERR () is not closed or no root

```


###  Fetch Detach
#### USA government tree
```
         |----------------------------------USA----------------------------------|
         |                                  |                                    |
    Legislature                      ExecutiveJudiciary                      Judiciary
   /         \                              |                                    |
House      Senate                      WhiteHouse                          SupremeCourt
 |            |                             |                                    |          
Pelosi      Harris                        Biden                               Roberts


127.0.0.1:6379> tree.init usa "USA (Legislature (House (Pelosi) Senate (Harris))ExecutiveJudiciary (WhiteHouse (Biden))Judiciary (SupremeCourt (Roberts)))"
OK

# Get subtree of executive judiciary
127.0.0.1:6379> tree.get_subtree usa ExecutiveJudiciary
"ExecutiveJudiciary( WhiteHouse( Biden ) )"

# Add secretary for Biden
127.0.0.1:6379> tree.set_subtree usa Biden "Blinken"
OK
# now biden has secretary
127.0.0.1:6379> tree.get_subtree usa Biden
"Biden( Blinken )"

# Detach Blinken from Biden
127.0.0.1:6379> tree.del_subtree usa Blinken
"Blinken"
127.0.0.1:6379> tree.get_subtree usa Biden
"Biden"


# Get Harris ancestors 
127.0.0.1:6379> tree.get_ancestors usa Harris
1) "Senate"
2) "Legislature"
3) "USA"

# Get Harris Father node
127.0.0.1:6379> tree.get_father usa Harris
"Senate"

# Get Legislature Children 
127.0.0.1:6379> tree.get_children usa Legislature
1) "House"
2) "Senate"


# Get Legislature Descendants(BFS)
127.0.0.1:6379>  tree.get_descendants usa  Legislature
1) "Legislature"
2) "House"
3) "Senate"
4) "Pelosi"
5) "Harris"


```

## Run
### Linux
```
redis-server --loadmodule yourpath/libretree.so
```

### Mac
```
redis-server --loadmodule ./target/debug/libretree.dylib
```

### Config
```
loadmodule /yourpath/libretree.so
```


## Dev
### Prerequisites
- Rust: Module written in rust
- Docker: For Cross Compile and Block box Test

### Makefile
```
build                          build retree docker image
builder                        build rust cross-compiler base image, for mac developers
clean                          clean
push                           push to docker hub
test                           do some behavioral tester
tester                         build tester image
```

### TODO
- Postgres ltree gist index
- Postgres ltree query
- Hash Index


## Thanks
- [RedisJSON](https://github.com/RedisJSON/RedisJSON)
- [trees](https://github.com/oooutlk/trees)
