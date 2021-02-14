FROM ohbonsai/retree:latest as tmp


FROM  python:3.6-slim-buster
COPY --from=tmp /usr/local/bin/redis-server /usr/local/bin/redis-server
COPY --from=tmp /usr/local/bin/redis-cli /usr/local/bin/redis-cli
COPY --from=tmp /usr/lib/redis/modules/retree.so /usr/lib/redis/modules/retree.so
COPY --from=tmp /data/redis.conf /etc/redis/redis.conf


# add python requirements
ADD ./tests/requirements.txt /tmp/requirements.txt
RUN pip install -r /tmp/requirements.txt --no-cache-dir -i https://mirrors.aliyun.com/pypi/simple


# set some default setting when testing
RUN mkdir -p /etc/redis \
    && sed -i 's/^\(daemonize .*\)$/# \1/' /etc/redis/redis.conf \
    && echo "\ndaemonize yes" >> /etc/redis/redis.conf \
    && sed -i 's/^\(logfile .*\)$/# \1/' /etc/redis/redis.conf \
    && sed -i 's/^\(Dir .*\)$/# \1/' /etc/redis/redis.conf \
    && echo "\nlogfile /tmp/redis-server.log" >> /etc/redis/redis.conf \
    && sed -i 's/^\(loglevel .*\)$/# \1/' /etc/redis/redis.conf \  
    && echo "\nloglevel notice" >> /etc/redis/redis.conf

RUN echo  "#!/bin/bash\n/usr/local/bin/redis-server /etc/redis/redis.conf --loadmodule /usr/lib/redis/modules/retree.so\npytest -s -v tests" > /run.sh
RUN chmod +x /run.sh

ENTRYPOINT ["/run.sh"]

