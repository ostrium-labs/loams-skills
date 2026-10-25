# BYOG runbook

- Owner: CHANGE_ME
- On-call: CHANGE_ME
- Cloud / region: CHANGE_ME
- VM / disk: CHANGE_ME
- Loams version: `0.1.1`
- Tailnet node: `loams.<tailnet>.ts.net`
- Bucket / KMS key: CHANGE_ME
- Secret store: CHANGE_ME

## Install

1. Create the VM with a local block disk and no inbound rule.
2. Attach the instance role / service account / managed identity with bucket and
   secret access only.
3. Enable bucket versioning. Record the KMS key.
4. Run the pinned installer from a tailnet machine:

   ```sh
   deploy/release/byoc/install.sh --cloud aws --bucket s3://CHANGE_ME \
     --region CHANGE_ME --version 0.1.1 --yes ubuntu@CHANGE_ME
   ```

5. Review the rendered `.env` with `--render-env-only` before a production run.
6. Confirm no `0.0.0.0` port is published and the cloud firewall has no inbound
   rule.
7. Open `https://loams.<tailnet>.ts.net/ui/` and complete setup with the setup
   token.
8. Create one API key per service; do not share keys between services.

## Verify

```sh
curl -fsS https://loams.<tailnet>.ts.net/healthz
docker compose ps
docker compose logs --since=10m loams
```

- Health 200.
- Every service `running`/`healthy`.
- Disk below 80%.
- Bucket versioning on.
- Last snapshot within the schedule.
- A known write is visible to a read with session consistency.

## Back up

- Disk snapshot on the cloud schedule; keep the previous snapshot through the
  upgrade window.
- Bucket versioning and lifecycle as the company policy requires.
- Record the snapshot id in the deployment record.

## Restore drill

1. Create a new VM and disk from the snapshot.
2. Attach the same bucket (or a copy) and the same instance role.
3. Install the same `LOAMS_VERSION`.
4. Point a test client at the restored tailnet node.
5. Verify counts and a known record.
6. Destroy the drill VM after recording the result.

Never restore over a live install. Restore into a new install and cut over.

## Upgrade

1. Read the release notes and the upgrade notes for every version between the
   current and target.
2. Snapshot the disk and confirm the bucket is versioned.
3. Pin the new `LOAMS_VERSION`; do not use `latest`.
4. Run the upstream upgrade path for the compose stack.
5. Verify health, a read, a write, and a search.
6. Keep the old snapshot until the next successful backup.

## Destroy

1. Export the records the company must keep.
2. Take a final snapshot if policy requires it.
3. `docker compose --profile '*' down --volumes` on the VM.
4. Delete the VM and disk.
5. Keep the bucket and snapshots for the retention period.
6. Revoke the tailnet node and the cloud identity.
