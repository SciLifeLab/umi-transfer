run:
	@cargo run --release -- --r1-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R1_001.fastq --r2-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R3_001.fastq separate --ru-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R2_001.fastq
run_in:
	@cargo run --release -- --r2-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R2.fastq --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq inline --pattern1 NNNNNNNNN --pattern2 NNNNNNNNN
clean:
	rm integrated*