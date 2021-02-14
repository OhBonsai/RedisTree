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
ADD ./redis.conf /data/redis.conf

# Set log path and not daemonize
RUN sed -i 's/^\(logfile .*\)$/# \1/' /data/redis.conf  \
    && echo "\nlogfile /data/redis-server.log" >> /data/redis.conf \
    && sed -i 's/^\(daemonize .*\)$/# \1/' /data/redis.conf \
    && echo "\ndaemonize no" >> /data/redis.conf

CMD ["redis-server", "/data/redis.conf", "--loadmodule", "/usr/lib/redis/modules/retree.so"]
