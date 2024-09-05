var adminDB = db.getSiblingDB('admin');
adminDB.auth('__ROOT_USERNAME__', '__ROOT_PASSWORD__');
rs.initiate({
    _id: 'rs0',
    members: [
        { _id: 0, host: 'localhost:27017' }
    ]
});
