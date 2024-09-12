-- insert 3 workspace
INSERT INTO workspaces(name, owner_id)
VALUES ('acme', 0),
       ('foo', 0),
       ('bar', 0);

-- insert 5 users, all with the same password
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

INSERT INTO messages(chat_id, sender_id, content)
VALUES (1, 1, 'Hello, world!'),
       (1, 2, 'Hi, there!'),
       (1, 3, 'How are you?'),
       (1, 4, 'I am fine, thank you!'),
       (1, 5, 'Good to hear that!'),
       (1, 1, 'Hello, world!'),
       (1, 2, 'Hi, there!'),
       (1, 3, 'How are you?'),
       (1, 1, 'Hello, world!'),
       (1, 1, 'Hello, world!');