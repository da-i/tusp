# tusp

Turbo task Spooler, more than `tsp` less than SLURM

## Planned usage

```bash
>tusp minimap2 -ax "map-ont" -o myalignment.bam myref.fasta myreads.fastq 
0
tusp -l
ID  CMD     STATUS  Time
0   mini... RUNNING 2s

// allow for 4 running processes
tusp -S 4
```

```bash
# terminal 1
tusp daemon

# terminal 2
tusp submit "echo hello from tusp"
tusp list
ID  CMD     STATUS   Time
1   echo... SUCCESS  2s

# IPC socket
# /tmp/tusp.sock
```


## Developer

Currently the daemon is blocked whist a process is executing

### responsibilities
1. job
Contain all the logic relevant to a single job.
1. repository
Keep track of all the jobs in circulation, also performs displays etc. keep track of job ids
1. scedualer
given a job and the fact that there is space on a node, decide on the next job to run on what node.
1. executor
Given a job and a node, execute the cmd and monitor and collect the result.
also provide info on node status.
1. daemon
Run in the background and orchestrate the above, handle IPC

`
