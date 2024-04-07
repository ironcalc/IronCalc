import type { Config } from "jest";
// import {defaults} from 'jest-config';

const config: Config = {
  // testMatch:["**.jest.mjs"],
  moduleFileExtensions: ["js", "ts", "mts", "mjs"],
  transform: {
    "^.+\\.[jt]s?$": "ts-jest",
  },
  moduleNameMapper: {
    "^@ironcalc/wasm$": "<rootDir>/node_modules/@ironcalc/nodejs/"
  },
};

export default config;
