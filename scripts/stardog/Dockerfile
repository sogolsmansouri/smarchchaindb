FROM bitnami/minideb:stretch

ENV STARDOG_HOME="/stardog"
ENV STARDOG_SERVER_JAVA_ARGS="-Xmx8g -XX:MaxDirectMemorySize=2g"

RUN install_packages curl openjdk-8-jre-headless unzip

RUN cd /opt && \
    curl -O 'http://stardog.download.s3-website-us-east-1.amazonaws.com/downloads/af8c58b9-9ab2-49aa-b133-55e11f666109/stardog-6.1.2.zip' && \
    unzip 'stardog-6.1.2.zip' && \
    mv 'stardog-6.1.2' 'stardog' && \
    rm 'stardog-6.1.2.zip'

RUN chmod +x /opt/stardog/bin/stardog-admin

# User data directory for $STARDOG_HOME
VOLUME ["/stardog"]

# Potential volume for license key to be lifted at runtime
VOLUME ["/stardog-license"]

COPY "stardog-license-key.bin" "/stardog-license/stardog-license-key.bin"

EXPOSE 5820

CMD rm -f "$STARDOG_HOME/system.lock" || true && \
    cp -f "/stardog-license/stardog-license-key.bin" "$STARDOG_HOME" 2>/dev/null || true && \
    /opt/stardog/bin/stardog-admin server start --foreground "$STARDOG_SERVER_FLAGS"

