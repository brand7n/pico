<?php

declare(strict_types=1);

namespace Remodulate\Pico;

/**
 * Mutable string builder for constructing output efficiently.
 */
class StringBuilder
{
    private string $buffer = '';

    public function append(PicoString $s): void
    {
        $this->buffer .= $s->raw();
    }

    public function appendChar(int $byte): void
    {
        $this->buffer .= chr($byte);
    }

    public function appendInt(int $val): void
    {
        $this->buffer .= (string) $val;
    }

    public function toString(): PicoString
    {
        return new PicoString($this->buffer);
    }

    public function length(): int
    {
        return strlen($this->buffer);
    }

    public function clear(): void
    {
        $this->buffer = '';
    }
}
