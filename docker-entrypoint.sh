#!/bin/bash
#
# Auth query entrypoint script, Switch redis mode by $1


err() {
  echo -e "[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $*" >&2
}

info() {
  echo -e "[$(date +'%Y-%m-%dT%H:%M:%S%z')]: $*"
}

function check_single_env() {
  info 'redis will start with single mode...'
  info "redis on port 6379"
}

# 和单点没有区别
function check_master_env() {
  if [[ -z "${REDIS_PORT}" ]]; then
    REDIS_PORT=6379
  fi
  sed -i 's/^\(port .*\)$/# \1/' /etc/redis/redis.conf
  echo -e "\nport $REDIS_PORT" >> /etc/redis/redis.conf

  info "redis will start with master mode..."
  info "redis on port $REDIS_PORT"
}

function check_slave_env() {
  if [[ -z "${REDIS_PORT}" ]]; then
    REDIS_PORT=6379
  fi
  sed -i 's/^\(port .*\)$/# \1/' /etc/redis/redis.conf
  echo -e "\nport $REDIS_PORT" >> /etc/redis/redis.conf

  if [[ -z "${MASTER_IP}" ]]; then
    err "you must provide mater ip"
    exit 1
  fi

  if [[ -z "${MASTER_PORT}" ]]; then
    err "master port set 6379"
    MASTER_PORT=6379
  fi

  sed -i 's/^\(slaveof .*\)$/# \1/' /etc/redis/redis.conf
  echo -e "\nslaveof $MASTER_IP $MASTER_PORT" >> /etc/redis/redis.conf

  if [[ -n "${MASTER_USER}" ]]; then
     sed -i 's/^\(masteruser .*\)$/# \1/' /etc/redis/redis.conf
     echo -e "\nmasteruser $MASTER_USER" >> /etc/redis/redis.conf
  fi

  if [[ -n "${MASTER_PASS}" ]]; then
     sed -i 's/^\(masterauth .*\)$/# \1/' /etc/redis/redis.conf
     echo -e "\nmasterauth $MASTER_PASS" >> /etc/redis/redis.conf
  fi

  info "redis will start with slave mode, master is $MASTER_IP:$MASTER_PORT"
  info "redis on port $REDIS_PORT"
}

function check_sentinel_env() {
  if [[ -z "${REDIS_PORT}" ]]; then
    REDIS_PORT=26379
  fi
  sed -i 's/^\(port .*\)$/# \1/' /etc/redis/sentinel.conf
  echo -e "\nport $REDIS_PORT" >> /etc/redis/sentinel.conf

  if [[ -z "${MASTER_IP}" ]]; then
    err "you must provide mater ip"
    exit 1
  fi

  if [[ -z "${MASTER_PORT}" ]]; then
    err "master port set 6379"
    MASTER_PORT=6379
  fi

  sed -i 's/^\(sentinel monitor mymaster.*\)$/# \1/' /etc/redis/sentinel.conf
  echo -e "\nsentinel monitor mymaster $MASTER_IP $MASTER_PORT 2" >> /etc/redis/sentinel.conf

  if [[ -n "${MASTER_USER}" ]]; then
     sed -i 's/^\(sentinel auth-user mymaster  .*\)$/# \1/' /etc/redis/sentinel.conf
     echo -e "\nsentinel auth-user mymaster  $MASTER_USER" >> /etc/redis/sentinel.conf
  fi

  if [[ -n "${MASTER_PASS}" ]]; then
     sed -i 's/^\(sentinel auth-pass mymaster .*\)$/# \1/' /etc/redis/sentinel.conf
     echo -e "\nsentinel auth-pass mymaster $MASTER_PASS" >> /etc/redis/sentinel.conf
  fi

  info "redis will start with sentinel mode, master is $MASTER_IP:$MASTER_PORT"
  info "redis on port $REDIS_PORT"
}

# shellcheck disable=SC1009
if [[ $1 == 'single' ]]; then
  check_single_env
  redis-server /etc/redis/redis.conf --loadmodule /usr/lib/redis/modules/retree.so /usr/lib/redis/modules/fdauth.so
  exit 1
elif [[ $1 == 'master' ]]; then
  check_master_env
  redis-server /etc/redis/redis.conf --loadmodule /usr/lib/redis/modules/retree.so /usr/lib/redis/modules/fdauth.so
  exit 1
elif [[ $1 == 'slave' ]]; then
  check_slave_env
  redis-server /etc/redis/redis.conf --loadmodule /usr/lib/redis/modules/retree.so /usr/lib/redis/modules/fdauth.so
  exit 1
elif [[ $1 == 'sentinel' ]]; then
  check_sentinel_env
  redis-server /etc/redis/sentinel.conf --loadmodule /usr/lib/redis/modules/retree.so /usr/lib/redis/modules/fdauth.so --sentinel
  exit 1
fi

exec "$@"