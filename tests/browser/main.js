//What Vite and webpack bundle.  The package is imported by its name, the way a project that uses a bundler does.

import * as advRandom from 'adv-random';
import { runChecks } from './checks.js';

window.__results = runChecks(advRandom);
