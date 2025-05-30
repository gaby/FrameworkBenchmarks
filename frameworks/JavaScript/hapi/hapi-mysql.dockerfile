FROM node:20.16-slim

COPY ./ ./

RUN npm install

ENV NODE_HANDLER sequelize

EXPOSE 8080

CMD ["node", "app.js"]
