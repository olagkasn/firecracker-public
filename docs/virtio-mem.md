### Boot with virtio-mem
For now there is no API endpoint for virtio-mem. Use the config file option with
the config_vm template (the template exists in repo root) as follows:
```
sudo ./firecracker --config-file config_vm
```