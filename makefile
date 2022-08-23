run:
	@cargo run --release -- --no-gzip --r1-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R1_001.fastq --r2-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R3_001.fastq separate --ru-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R2_001.fastq
run_in:
	@cargo run --release -- --no-gzip --r2-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R2.fastq --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq inline --pattern1 NNNNNNNNN --pattern2 NNNNNNNNN
clean:
	rm -f integrated*
	rm run*
run_gz:
	@cargo run --release -- --no-gzip --r1-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R1_001.fastq.gz  --r2-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R3_001.fastq.gz separate --ru-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R2_001.fastq.gz
run_in_gz:
	@cargo run --release -- --no-gzip --r2-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R2.fastq.gz --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq.gz inline --pattern1 NNNNNNNNN --pattern2 NNNNNNNNN
run_in_gz_sing:
	@cargo run --release -- --no-gzip --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq.gz inline --pattern1 NNNNNNNNN
run_all:
	@cargo run --release -- --no-gzip --prefix run --r1-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R1_001.fastq --r2-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R3_001.fastq separate --ru-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R2_001.fastq
	@cargo run --release -- --no-gzip --prefix run_in --r2-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R2.fastq --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq inline --pattern1 NNNNNNNNN --pattern2 NNNNNNNNN
	@cargo run --release -- --no-gzip --prefix run_gz --r1-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R1_001.fastq.gz  --r2-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R3_001.fastq.gz separate --ru-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R2_001.fastq.gz
	@cargo run --release -- --no-gzip --prefix run_in_gz --r2-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R2.fastq.gz --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq.gz inline --pattern1 NNNNNNNNN --pattern2 NNNNNNNNN
	@cargo run --release -- --no-gzip --prefix run_in_gz_sing --r1-in /Users/judit/Downloads/barcodex-rs-master/inline-test/test_fil_R1.fastq.gz inline --pattern1 NNNNNNNNN

run_test:
	@cargo run --release -- --no-gzip --prefix run --r1-in /Users/judit/UMIembed/fil/P21809_150_S28_L001_R1_001.fastq  help
