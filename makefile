run:
	@cargo run --release -- --plain --r1-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R1_001.fastq --r2-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R3_001.fastq separate --ru-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R2_001.fastq
run_in:
	@cargo run --release -- --plain --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq --r2-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R2.fastq inline --pattern1 NNNNNNNNN --pattern2 NNNNNNNNN
run_gz:
	@cargo run --release -- --plain --r1-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R1_001.fastq.gz --r2-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R3_001.fastq.gz separate --ru-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R2_001.fastq.gz
run_in_gz:
	@cargo run --release -- --plain --r2-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R2.fastq.gz --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq.gz inline --pattern1 NNNNNNNNN --pattern2 NNNNNNNNN
run_sing:
	@cargo run --release -- --plain --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq inline --pattern1 NNNNNNNNN
run_sing_gz:
	@cargo run --release -- --plain --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq.gz inline --pattern1 NNNNNNNNNN
run_in_test:
	@cargo run --release -- --plain --r1-out R1_out --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq --r2-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R2.fastq inline --pattern1 NNNNNNNNN
clean:
	rm -f integrated*