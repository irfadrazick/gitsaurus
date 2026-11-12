import { expect, it } from 'vitest';
import { forcePushBody, forcePushWarning } from './push';

it('warns that a force push overwrites remote history', () => {
  expect(forcePushWarning('main')).toBe('The remote has commits main lacks. Force push (with lease) overwrites remote history.');
});

it('puts the warning above git stderr', () => {
  expect(forcePushBody('main', ' ! [rejected] main -> main (fetch first)\n')).toBe(
    `${forcePushWarning('main')}\n\n! [rejected] main -> main (fetch first)`,
  );
  expect(forcePushBody('main', '  ')).toBe(forcePushWarning('main'));
});
