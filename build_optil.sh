podman build . -t optil
podman rm optilbin
podman create --name optilbin localhost/optil:latest
mkdir optil_target/
rm optil_target/dfvstritus
podman cp optilbin:./target/release/dfvstritus optil_target/
tar -czvf optil_target/dfvstritus.tgz -C optil_target/ dfvstritus