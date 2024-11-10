import fs from 'fs';
import { exec } from 'child_process';

// read changesets version info
async function getNewVersion() {
  return new Promise((resolve, reject) => {
    exec('node_modules/.bin/changeset status', (error, stdout) => {
      if (error) {
        reject(error);
        return;
      }

      const versionMatch = stdout.match(/Version: (.+)/);
      if (versionMatch) {
        resolve(versionMatch[1].trim());
      } else {
        resolve(null);
      }
    });
  });
}

async function main() {
  const newVersion = await getNewVersion();
  if (!newVersion) return;

  // read Cargo.toml
  const cargoPath = './Cargo.toml';
  let cargoContent = fs.readFileSync(cargoPath, 'utf8');

  // update version
  cargoContent = cargoContent.replace(
    /version = "(.*?)"/,
    `version = "${newVersion}"`
  );

  // write back to file
  fs.writeFileSync(cargoPath, cargoContent);
}

main().catch(console.error);
