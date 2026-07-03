#!/usr/bin/env node
// CI 装配脚本:读取 npm/ 下的 package.json 模板,注入版本号/平台信息,
// 输出到 $OUT_DIR/package.json (默认 pkg/) 供 `npm publish` 使用。
//
//   主包: VERSION=0.1.3 node npm/build-package.mjs main
//   子包: VERSION=0.1.3 SUB=linux-x64 NPM_OS=linux NPM_CPU=x64 BIN=nuwax-upload \
//         node npm/build-package.mjs sub
//
// 失败快速:任何必需环境变量缺失即抛错退出(Fail Fast)。

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const mode = process.argv[2];

const version = process.env.VERSION;
if (!version) {
  console.error('build-package: VERSION env is required');
  process.exit(1);
}

const outDir = process.env.OUT_DIR ?? 'pkg';
mkdirSync(outDir, { recursive: true });

function render(templateFile, replacements) {
  const tpl = readFileSync(join(here, templateFile), 'utf8');
  let out = tpl;
  for (const [k, v] of Object.entries(replacements)) {
    out = out.replaceAll(k, v);
  }
  return out;
}

if (mode === 'main') {
  const out = render('main-package.json', { __VERSION__: version });
  writeFileSync(join(outDir, 'package.json'), out);
} else if (mode === 'sub') {
  const { SUB, NPM_OS, NPM_CPU, BIN } = process.env;
  for (const [k, v] of [['SUB', SUB], ['NPM_OS', NPM_OS], ['NPM_CPU', NPM_CPU], ['BIN', BIN]]) {
    if (!v) {
      console.error(`build-package(sub): ${k} env is required`);
      process.exit(1);
    }
  }
  const out = render('sub-package.json', {
    __NAME__: `nuwax-upload-${SUB}`,
    __VERSION__: version,
    __OS__: NPM_OS,
    __CPU__: NPM_CPU,
    __FILE__: BIN,
  });
  writeFileSync(join(outDir, 'package.json'), out);
} else {
  console.error('build-package: mode must be "main" or "sub"');
  process.exit(1);
}

console.log(`build-package: wrote ${join(outDir, 'package.json')}`);
