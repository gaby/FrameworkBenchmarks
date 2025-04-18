FROM maven:3.9.0-eclipse-temurin-17 as maven
WORKDIR /vertx
COPY src src
COPY pom.xml pom.xml
RUN mvn package -q

EXPOSE 8080

CMD export DBIP=`getent hosts tfb-database | awk '{ print $1 }'` && \
    sed -i "s|tfb-database|$DBIP|g" /vertx/src/main/conf/config.json && \
    java \
      -Xms2G \
      -Xmx2G \
      -server \
      -XX:+UseNUMA \
      -XX:+UseParallelGC \
      -Djava.lang.Integer.IntegerCache.high=10000 \
      -Dvertx.disableMetrics=true \
      -Dvertx.disableWebsockets=true \
      -Dvertx.disableContextTimings=true \
      -Dvertx.disableHttpHeadersValidation=true \
      -Dvertx.cacheImmutableHttpResponseHeaders=true \
      -Dvertx.internCommonHttpRequestHeadersToLowerCase=true \
      -Dvertx.eventLoopPoolSize=$((`grep --count ^processor /proc/cpuinfo`)) \
      -Dio.netty.buffer.checkBounds=false  \
      -Dio.netty.buffer.checkAccessible=false \
      -jar \
      target/vertx.benchmark-0.0.1-SNAPSHOT-fat.jar \
      src/main/conf/config.json
