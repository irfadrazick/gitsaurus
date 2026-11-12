/** The force-push confirm text, shared by the branch context menu and the toolbar. */
export const forcePushWarning = (branch: string) =>
  `The remote has commits ${branch} lacks. Force push (with lease) overwrites remote history.`;

/** Confirm body: the warning, then git's rejection output when there is any. */
export function forcePushBody(branch: string, stderr: string) {
  const detail = stderr.trim();
  return detail ? `${forcePushWarning(branch)}\n\n${detail}` : forcePushWarning(branch);
}
