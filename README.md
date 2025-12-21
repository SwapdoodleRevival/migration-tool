# Swapdoodle Migration Tool

This utility will read your Swapdoodle extdata and assist you in migrating your Notes from Nintendo to Pretendo.

## Why is this needed? 

Swapdoodle attributes your Swapdoodle Notes using something called a PID. The PID is a user's unique identifier in the Friends database. When you add a Friend on your 3DS, their PID is stored on your console.

When you switch to Pretendo Network and add Friends on Pretendo, their PID will be *different* from the one they had on Nintendo Network. This means your Swapdoodle Notes will no longer be mapped to their sender correctly and thus end up in the **Unknown Sender** category.

The tool will read your extdata and automatically attempt to match your old Nintendo Network Friends to your new Pretendo Network Friends. It is based on the MAC Address stored in Mii Data, which is available in both the Friend List and every Swapdoodle Note. This should do the trick for 99% of users, but the tool allows you to fine-tune the mapping in case you need to map something else.

## Status

We've tested this tool on two devices, where we've confirmed it to work. It should work regardless of how many notes you have. Still, random crashes and bugs may occur. If you encounter any bugs or crashes, please tell us!

> [!WARNING]
> This tool currently **does not back up your extdata** before processing it. 
> 
> You are advised to back up your **extdata** (*not* save data!) before using this tool. Checkpoint should be installed on nearly all homebrewed 3DS devices and works best for this task.

## Download

[Download the Alpha from the Releases page.](https://github.com/SwapdoodleRevival/migration-tool/releases/)