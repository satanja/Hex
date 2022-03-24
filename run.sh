cargo build --release
cd ./unsolved
for filename in *; do 
    ../target/release/dfvstritus < "$filename" > ../kernels/k"$filename"
done