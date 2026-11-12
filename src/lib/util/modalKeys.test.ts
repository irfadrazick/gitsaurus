import { expect, it } from 'vitest';
import { shouldEnterConfirm } from './modalKeys';

it('enter confirms only plain confirms without a focused button', () => {
  expect(shouldEnterConfirm('confirm', false, 'BODY')).toBe(true);
  expect(shouldEnterConfirm('confirm', false, null)).toBe(true);
  expect(shouldEnterConfirm('confirm', false, 'BUTTON')).toBe(false);
  expect(shouldEnterConfirm('confirm', true, 'BODY')).toBe(false);
  expect(shouldEnterConfirm('prompt', false, 'BODY')).toBe(false);
  expect(shouldEnterConfirm('choose', false, 'BODY')).toBe(false);
});
