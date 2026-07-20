# LVM Management Reference

## LVM Architecture

```
Physical Volume (PV) → Volume Group (VG) → Logical Volume (LV)
```

## Creating LVM from Scratch

### 1. Create Physical Volume
```bash
pvcreate /dev/sdb
pvdisplay
```

### 2. Create Volume Group
```bash
vgcreate vg_data /dev/sdb
vgdisplay
```

### 3. Create Logical Volume
```bash
# Create 100G LV named 'data'
lvcreate -L 100G -n data vg_data

# Create LV using all remaining space
lvcreate -l 100%FREE -n data vg_data
```

### 4. Format and Mount
```bash
mkfs.ext4 /dev/vg_data/data
mkdir -p /mnt/data
mount /dev/vg_data/data /mnt/data
echo '/dev/vg_data/data /mnt/data ext4 defaults 0 2' >> /etc/fstab
```

## Resizing LVM

### Extend Logical Volume
```bash
# Resize LV to 200G
lvextend -L 200G /dev/vg_data/data

# Resize filesystem (ext4)
resize2fs /dev/vg_data/data
```

### Reduce Logical Volume (EXT4 - must unmount first)
```bash
umount /mnt/data
e2fsck -f /dev/vg_data/data
resize2fs /dev/vg_data/data 50G
lvreduce -L 50G /dev/vg_data/data
mount /dev/vg_data/data /mnt/data
```

### Extend Volume Group
```bash
# Add new disk to VG
pvcreate /dev/sdc
vgextend vg_data /dev/sdc

# Now LV can be extended beyond original disk size
```

## Common Operations

### Snapshot
```bash
# Create 10G snapshot
lvcreate -L 10G -s -n data_snapshot /dev/vg_data/data

# Mount snapshot for read-only access
mount -o ro /dev/vg_data/data_snapshot /mnt/snapshot
```

### Remove LVM
```bash
umount /mnt/data
lvremove /dev/vg_data/data
vgremove vg_data
pvremove /dev/sdb
```

## Troubleshooting

### VG not found
```bash
vgcfgrestore vg_data
vgchange -ay vg_data
```

### PV missing
```bash
pvscan
pvck /dev/sdX
vgreduce --removemissing vg_data
```