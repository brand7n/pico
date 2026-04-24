<?php

declare(strict_types=1);

namespace Remodulate\Pico;

/**
 * Static file I/O. Read or write entire files.
 */
class File
{
    public static function readContents(PicoString $path): PicoString
    {
        $data = file_get_contents($path->raw());
        if ($data === false) {
            throw new \RuntimeException("File::readContents — failed to read: {$path->raw()}");
        }
        return new PicoString($data);
    }

    public static function writeContents(PicoString $path, PicoString $data): void
    {
        $result = file_put_contents($path->raw(), $data->raw());
        if ($result === false) {
            throw new \RuntimeException("File::writeContents — failed to write: {$path->raw()}");
        }
    }

    public static function exists(PicoString $path): bool
    {
        return file_exists($path->raw());
    }

    public static function isFile(PicoString $path): bool
    {
        return is_file($path->raw());
    }

    public static function isDir(PicoString $path): bool
    {
        return is_dir($path->raw());
    }
}
