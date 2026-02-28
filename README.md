# tusp

Turbo task Spooler, more than `tsp` less than SLURM

## Planned usage

`bash
>tusp minimap2 -ax "map-ont" -o myalignment.bam myref.fasta myreads.fastq 
0
tusp -l
ID  CMD     STATUS  Time
0   mini... RUNNING 2s

// allow for 4 running processes
tusp -S 4
`
