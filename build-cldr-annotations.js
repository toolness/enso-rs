//@ts-check

const fs = require('fs');
const path = require('path');

const JSON_FILENAME = 'annotations.json';
const RS_FILENAME = path.join('src', 'commands', 'cldr_annotations.rs');

if (!fs.existsSync(JSON_FILENAME)) {
  console.log(`Please download the JSON version of the Unicode CLDR annotations and save it in the current directory as ${JSON_FILENAME}.`);
  console.log('You can download it from: https://raw.githubusercontent.com/unicode-org/cldr-json/master/cldr-json/cldr-annotations-full/annotations/en/annotations.json');
  process.exit(1);
}

const unknownChars = new Set();

/**
 * @arg count {number}
 * @returns {String}
 */
function spaces(count) {
  let spaces = [];

  for (let i = 0; i < count; i++) {
    spaces.push(' ');
  }

  return spaces.join('');
}

/**
 * @arg value {String}
 * @returns {String}
 */
function formatForCommandName(value) {
  value = value
    .toLocaleLowerCase()
    .replace(/[“”]/g, '"')
    .replace(/’/g, "'")
    .replace(/ñ/g, "n");

  for (let i = 0; i < value.length; i++) {
    let codePoint = value.codePointAt(i);
    if (codePoint !== undefined && (codePoint < 32 || codePoint > 126)) {
      if (!unknownChars.has(codePoint)) {
        unknownChars.add(codePoint);
        console.warn(`WARNING: Unicode codepoint ${codePoint} is not currently supported by Enso:`);
        console.warn(`  ${value}`);
        console.warn(`  ${spaces(i)}^\n`);
      }
    }
  }
  return value;
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

for (let [char, item] of Object.entries(data.annotations.annotations)) {
  const name = formatForCommandName(item.tts[0]);
  lines.push(`  (${JSON.stringify(char)}, ${JSON.stringify(name)}),`);
}

lines.push('];\n');

fs.writeFileSync(RS_FILENAME, lines.join('\n'), { encoding: 'utf-8' });

console.log(`Wrote ${RS_FILENAME}.`);
