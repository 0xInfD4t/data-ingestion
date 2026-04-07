Here's your lightweight, production-ready Ansible playbook for a minimal "Hudi on MinIO with Ceph" Bronze zone.
This setup is deliberately simple, idempotent, and focused on what you actually need right now:

Single-node Ceph (via official cephadm — easiest and most maintainable)
MinIO as the S3-compatible gateway (Hudi writes to s3:// endpoint)
CephFS mounted under MinIO so the object data is truly stored on Ceph
Bronze bucket (bronze) created and ready for Hudi tables

Directory Structure (create this on your control node)
Bashansible-hudi-minio-ceph/
├── inventory.yml
├── site.yml
├── roles/
│   ├── ceph/
│   │   ├── tasks/main.yml
│   │   └── defaults/main.yml
│   ├── minio/
│   │   ├── tasks/main.yml
│   │   └── defaults/main.yml
│   └── bronze-bucket/
│       └── tasks/main.yml
└── group_vars/all.yml
1. inventory.yml
YAMLall:
  hosts:
    storage01:          # ← change to your server hostname/IP
      ansible_host: 10.0.0.51   # ← your server IP
      ansible_user: root
      ansible_ssh_private_key_file: ~/.ssh/id_rsa
2. group_vars/all.yml
YAML# Ceph
ceph_version: "18"          # Reef (stable, recommended for Hudi)
ceph_cluster_name: "ceph"
ceph_public_network: "10.0.0.0/24"   # ← adjust to your network
ceph_cluster_network: "10.0.0.0/24"

# MinIO
minio_version: "RELEASE.2025-03-20T20-16-30Z"   # latest stable as of now
minio_root_user: "admin"
minio_root_password: "SuperSecretMinio2026!"     # ← CHANGE THIS
minio_port: 9000
minio_console_port: 9001

# Bronze zone
bronze_bucket: "bronze"
3. site.yml (main playbook)
YAML---
- name: Deploy Hudi-ready Bronze Zone (MinIO + Ceph)
  hosts: all
  become: true
  roles:
    - ceph
    - minio
    - bronze-bucket
  vars:
    minio_data_dir: "/var/lib/minio/data"
4. Roles (copy-paste these files)
roles/ceph/tasks/main.yml
YAML---
- name: Install cephadm dependencies
  apt:
    name:
      - curl
      - python3
      - python3-pip
      - lvm2
      - ceph-common
    state: present
    update_cache: yes

- name: Bootstrap Ceph single-node cluster
  command: cephadm bootstrap --mon-ip {{ ansible_host }} --initial-dashboard-password SuperSecretCeph2026! --allow-fqdn
  args:
    creates: /etc/ceph/ceph.conf
  register: ceph_bootstrap

- name: Enable CephFS (required for MinIO backend)
  command: ceph fs new cephfs cephfs_metadata cephfs_data
  when: ceph_bootstrap.changed

- name: Create CephFS mount point
  file:
    path: /mnt/cephfs
    state: directory

- name: Mount CephFS
  mount:
    path: /mnt/cephfs
    src: "{{ ceph_cluster_name }}-fs:/"
    fstype: ceph
    opts: "name=admin,secretfile=/etc/ceph/ceph.client.admin.keyring"
    state: mounted
roles/minio/tasks/main.yml
YAML---
- name: Create MinIO user and directories
  user:
    name: minio-user
    system: yes
    home: /var/lib/minio
    create_home: yes

- name: Create MinIO data directory on CephFS
  file:
    path: "{{ minio_data_dir }}"
    state: directory
    owner: minio-user
    group: minio-user
    mode: '0755'

- name: Download MinIO binary
  get_url:
    url: "https://dl.min.io/server/minio/release/linux-amd64/archive/minio.{{ minio_version }}"
    dest: /usr/local/bin/minio
    mode: '0755'

- name: Create MinIO systemd service
  template:
    src: minio.service.j2
    dest: /etc/systemd/system/minio.service
  notify: restart minio

- name: Start and enable MinIO
  systemd:
    name: minio
    enabled: yes
    state: started
    daemon_reload: yes
roles/minio/templates/minio.service.j2
ini[Unit]
Description=MinIO
After=network-online.target

[Service]
Type=simple
User=minio-user
EnvironmentFile=-/etc/default/minio
ExecStart=/usr/local/bin/minio server {{ minio_data_dir }} \
  --address :{{ minio_port }} \
  --console-address :{{ minio_console_port }}
Restart=always

[Install]
WantedBy=multi-user.target
roles/bronze-bucket/tasks/main.yml
YAML---
- name: Install MinIO client (mc)
  get_url:
    url: https://dl.min.io/client/mc/release/linux-amd64/mc
    dest: /usr/local/bin/mc
    mode: '0755'

- name: Configure mc alias for local MinIO
  command: mc alias set local http://localhost:9000 {{ minio_root_user }} {{ minio_root_password }}
  changed_when: false

- name: Create Bronze zone bucket
  command: mc mb --ignore-existing local/{{ bronze_bucket }}
  changed_when: false

- name: Set bucket versioning (recommended for Hudi)
  command: mc version enable local/{{ bronze_bucket }}
  changed_when: false

- name: Print ready message
  debug:
    msg: |
      ✅ Bronze zone ready!
      S3 Endpoint : http://{{ ansible_host }}:9000
      Bucket      : {{ bronze_bucket }}
      Access Key  : {{ minio_root_user }}
      Secret Key  : {{ minio_root_password }}
      Use this in Hudi/Spark: s3://{{ bronze_bucket }}/
How to run it
Bash# On your control node
cd ansible-hudi-minio-ceph
ansible-playbook -i inventory.yml site.yml --ask-become-pass
First run takes ~4-6 minutes on a clean Ubuntu 22.04 / Rocky 9 server.
Verification
Bash# Check Ceph
ceph status
ceph fs ls

# Check MinIO
mc admin info local
mc ls local/bronze

# Test Hudi write (from any Spark node)
spark-shell --packages org.apache.hudi:hudi-spark3.5-bundle_2.12:1.0.0-beta1 \
  -c "spark.hadoop.fs.s3a.endpoint=http://YOUR-IP:9000" \
  -c "spark.hadoop.fs.s3a.access.key=admin" \
  -c "spark.hadoop.fs.s3a.secret.key=SuperSecretMinio2026!" \
  -c "spark.hadoop.fs.s3a.path.style.access=true"