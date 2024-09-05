sed -i "s/__ROOT_USERNAME__/${MONGO_INITDB_ROOT_USERNAME}/g" /mongo-init.js
sed -i "s/__ROOT_PASSWORD__/${MONGO_INITDB_ROOT_PASSWORD}/g" /mongo-init.js
mongosh --file /mongo-init.js
rm /mongo-init.js
echo "done"