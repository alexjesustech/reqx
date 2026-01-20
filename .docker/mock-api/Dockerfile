FROM harbor.local:5000/dockerhub-cache/library/node:20-alpine

WORKDIR /app

COPY package.json server.js ./

RUN npm install --production

EXPOSE 3000

HEALTHCHECK --interval=5s --timeout=3s --start-period=5s \
    CMD wget -qO- http://localhost:3000/health || exit 1

CMD ["node", "server.js"]
