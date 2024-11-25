-- Your SQL goes here



ALTER TABLE `user` DROP COLUMN `groups`;
ALTER TABLE `user` ADD COLUMN `groups` USERGROUPSSET(5);

