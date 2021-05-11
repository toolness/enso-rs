const fs = require('fs');
const path = require('path');

const JSON_FILENAME = 'annotations.json';
const RS_FILENAME = path.join('src', 'cldr_annotations.rs');

if (!fs.existsSync(JSON_FILENAME)) {
  console.log(`Please download the JSON version of the Unicode CLDR annotations and save it in the current directory as ${JSON_FILENAME}.`);
  console.log('You can download it from: https://github.com/unicode-org/cldr-json/blob/master/cldr-json/cldr-annotations-full/annotations/en/annotations.json');
  process.exit(1);
}

const data = JSON.parse(fs.readFileSync('annotations.json', { encoding: 'utf-8' }));

const numEntries = Array.from(Object.keys(data.annotations.annotations)).length;

const ver = data.annotations.identity.version._cldrVersion;

const lines = [
  `// This data was auto-generated from the Unicode CLDR version ${ver}.`,
  `// Please do not edit it.`,
  '',
  `pub const CLDR_ANNOTATIONS: [(&'static str, &'static str); ${numEntries}] = [`
];

for ([char, item] of Object.entries(data.annotations.annotations)) {
  lines.push(`  (${JSON.stringify(char)}, "${item.tts[0]}"),`);
}

lines.push('];\n');

fs.writeFileSync(RS_FILENAME, lines.join('\n'), { encoding: 'utf-8' });

console.log(`Wrote ${RS_FILENAME}.`);
