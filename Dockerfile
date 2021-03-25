FROM ohbonsai/retree-builder:latest as builder

ENV LIBDIR /usr/lib/redis/modules
ADD . /RETREE
WORKDIR /RETREE

# Build the source
RUN set -ex ;\
    cargo build --release ;\
    mv target/release/libretree.so target/release/retree.so


#----------------------------------------------------------------------------------------------
# Package the runner
FROM redis:6.0.10

WORKDIR /data
COPY --from=builder /RETREE/target/release/retree.so /usr/lib/redis/modules/retree.so
ADD ./redis.conf /etc/redis/redis.conf
ADD ./sentinel.conf /etc/redis/sentinel.conf
ADD ./acl.file /etc/redis/acl.file
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod a+x /docker-entrypoint.sh

WORKDIR /data

# set log path
# set aclfile
# set daemonize no
RUN sed -i 's/^\(logfile .*\)$/# \1/' /etc/redis/redis.conf  \
    && echo "\nlogfile /data/redis-server.log" >> /etc/redis/redis.conf \
    && sed -i 's/^\(daemonize .*\)$/# \1/' /etc/redis/redis.conf \
    && echo "\ndaemonize no" >> /etc/redis/redis.conf \
    && echo "\naclfile /etc/redis/acl.file" >> /etc/redis/redis.conf \
    && sed -i 's/^\(user .*\)$/# \1/' /etc/redis/redis.conf



# Load the entrypoint script to be run later
ENTRYPOINT ["/docker-entrypoint.sh"]

# Invoke the entrypoint script
CMD ["single"]
