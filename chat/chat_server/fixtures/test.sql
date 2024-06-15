-- insert 3 workspace
INSERT INTO workspaces(name, owner_id)
VALUES ('acme', 0),
       ('foo', 0),
       ('bar', 0);

-- insert 5 users
INSERT INTO users(fullname, email, password_hash, ws_id)
VALUES ('Alice',
        'alice@acme.org',
        '$argon2id$v=19$m=19456,t=2,p=1$SAfkPh4FHkLIWKwUXTfv5A$0mgR5YSCfwvBi44UecVGQ6/mxzKvG76HRCm5ovJcjz8',
        1),
       ('Bob',
        'bob@acme.org',
        '$argon2id$v=19$m=19456,t=2,p=1$SAfkPh4FHkLIWKwUXTfv5A$0mgR5YSCfwvBi44UecVGQ6/mxzKvG76HRCm5ovJcjz8',
        1),
       ('Charlie',
        'charlie@acme.org',
        '$argon2id$v=19$m=19456,t=2,p=1$SAfkPh4FHkLIWKwUXTfv5A$0mgR5YSCfwvBi44UecVGQ6/mxzKvG76HRCm5ovJcjz8',
        1),
       ('David',
        'david@acme.org',
        '$argon2id$v=19$m=19456,t=2,p=1$SAfkPh4FHkLIWKwUXTfv5A$0mgR5YSCfwvBi44UecVGQ6/mxzKvG76HRCm5ovJcjz8',
        1),
       ('Eve',
        'eve@acme.org',
        '$argon2id$v=19$m=19456,t=2,p=1$SAfkPh4FHkLIWKwUXTfv5A$0mgR5YSCfwvBi44UecVGQ6/mxzKvG76HRCm5ovJcjz8',
        1);

-- insert 4 chats
-- insert public/private channel
INSERT INTO chats(ws_id, name, type, members)
VALUES (1, 'general', 'public_channel', '{1,2,3,4,5}'),
       (1, 'private', 'private_channel', '{1,2,3}');

-- insert unnamed chat
INSERT INTO chats(ws_id, type, members)
VALUES (1, 'group', '{1,2,3,4}'),
       (1, 'single', '{1,2}');