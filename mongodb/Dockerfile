FROM mongo:latest

COPY generate-keyfile.sh /generate-keyfile.sh
COPY mongo-init.js /mongo-init.js 
COPY init.sh /init.sh
RUN chmod +x /init.sh

RUN mkdir -p /opt/keyfile && \
    /bin/bash /generate-keyfile.sh
CMD ["mongod", "--replSet", "rs0",  "--keyFile", "/opt/keyfile/mongo-keyfile"]