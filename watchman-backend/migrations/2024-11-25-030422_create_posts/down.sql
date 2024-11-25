-- This file should undo anything in `up.sql`



ALTER TABLE `user` DROP COLUMN `groups`;
ALTER TABLE `user` ADD COLUMN `groups` SET(5);

