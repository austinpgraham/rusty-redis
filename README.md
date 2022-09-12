# Rusty-Redis
This is a simple CLI application to help with managing local Redis clusters...because let's be honest it's a pain.

## Usage
You can also access simple usage notes by running `rr --help`. As of release 0.1.0 (the initial release), there are three commands available:

`rr config ls` with an optional --base-dir flag: list all the Redis cluster config files you currently have created.
`rr cluster start`: start all nodes with a configuration file and create the Redis cluster.
`rr cluster stop`: stop all processes on your current cluster.
