#!/usr/bin/env python3
"""Offline integration tests: python3 .github/scripts/test-docs-publish.py."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).with_name('docs-publish').resolve()


class PublishTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.remote = self.root / 'remote.git'
        self.repo = self.root / 'repo'
        self.repo.mkdir()
        self.env = dict(os.environ, GIT_CONFIG_NOSYSTEM='1',
                        GIT_CONFIG_GLOBAL=os.devnull,
                        GIT_AUTHOR_NAME='Test', GIT_AUTHOR_EMAIL='test@example.com',
                        GIT_COMMITTER_NAME='Test', GIT_COMMITTER_EMAIL='test@example.com')
        self.run_git('init', '--bare', str(self.remote))
        self.run_git('init', '-b', 'gh-pages')
        self.run_git('remote', 'add', 'origin', str(self.remote))
        (self.repo / 'latest').mkdir()
        (self.repo / 'latest/index.html').write_text('release')
        (self.repo / 'starlight').mkdir()
        (self.repo / 'starlight/index.html').write_text('old')
        self.run_git('add', '.')
        self.run_git('commit', '-m', 'Initial docs')
        self.run_git('push', 'origin', 'gh-pages')
        self.initial = self.remote_head()
        # Track publication events, not just the number of generated commits.
        hook = self.remote / 'hooks/post-receive'
        hook.write_text('#!/bin/sh\necho push >> "' + str(self.root / 'pushes') + '"\n')
        hook.chmod(0o755)
        self.run_git('checkout', '-b', 'main')

    def run_git(self, *args):
        return subprocess.check_output(['git', *args], cwd=self.repo, env=self.env,
                                       stderr=subprocess.DEVNULL, text=True).strip()

    def remote_head(self):
        return self.run_git('--git-dir=' + str(self.remote), 'rev-parse', 'gh-pages')

    def prepare_mkdocs(self, empty=False):
        self.run_git('checkout', 'gh-pages')
        if not empty:
            (self.repo / 'prerelease').mkdir(exist_ok=True)
            (self.repo / 'prerelease/index.html').write_text('new docs')
            self.run_git('add', '.')
        self.run_git('commit', '--allow-empty', '-m', 'mike deploy prerelease')
        self.run_git('checkout', 'main')

    def publish(self, *args):
        return subprocess.run(['sh', str(SCRIPT), *args], cwd=self.repo,
                              env=self.env, capture_output=True, text=True)

    def assert_pushes(self, count):
        log = self.root / 'pushes'
        self.assertEqual(len(log.read_text().splitlines()) if log.exists() else 0, count)

    def test_unchanged_output(self):
        self.assertEqual(self.publish().returncode, 0)
        self.assertEqual(self.remote_head(), self.initial)
        self.assert_pushes(0)

    def test_empty_mike_commit(self):
        self.prepare_mkdocs(empty=True)
        self.assertEqual(self.publish().returncode, 0)
        self.assertEqual(self.remote_head(), self.initial)
        self.assert_pushes(0)

    def test_mkdocs_only_retains_starlight_and_release(self):
        self.prepare_mkdocs()
        self.assertEqual(self.publish().returncode, 0)
        self.assertEqual(self.run_git('show', 'origin/gh-pages:starlight/index.html'), 'old')
        self.assertEqual(self.run_git('show', 'origin/gh-pages:latest/index.html'), 'release')
        self.assert_pushes(1)

    def test_combined_publication_and_repeat(self):
        self.prepare_mkdocs()
        build = self.root / 'built site'
        build.mkdir()
        (build / 'index.html').write_text('new starlight')
        result = self.publish(str(build))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.run_git('show', 'origin/gh-pages:starlight/index.html'), 'new starlight')
        self.assertEqual(self.run_git('show', 'origin/gh-pages:prerelease/index.html'), 'new docs')
        self.assertEqual(self.run_git('show', 'origin/gh-pages:latest/index.html'), 'release')
        self.assert_pushes(1)
        self.assertEqual(self.publish(str(build)).returncode, 0)
        self.assert_pushes(1)

    def test_missing_build_does_not_publish(self):
        self.prepare_mkdocs()
        self.assertNotEqual(self.publish('missing').returncode, 0)
        self.assert_pushes(0)

    def test_concurrent_release_is_not_overwritten(self):
        self.prepare_mkdocs()
        # Advance the remote independently, without updating this checkout's tracking ref.
        tree = self.run_git('rev-parse', self.initial + '^{tree}')
        commit = self.run_git('commit-tree', tree, '-p', self.initial, '-m', 'Concurrent release')
        self.run_git('--git-dir=' + str(self.remote), 'fetch', str(self.repo), commit)
        self.run_git('--git-dir=' + str(self.remote), 'update-ref', 'refs/heads/gh-pages', commit)
        self.assertNotEqual(self.publish().returncode, 0)
        self.assertEqual(self.remote_head(), commit)
        self.assert_pushes(0)


if __name__ == '__main__':
    unittest.main()
